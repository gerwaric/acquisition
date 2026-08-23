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

## Preconditions (owner decisions, 2026-08-22)

- **Exclusive use.** No other Acquisition instance — the shipped C++ app, a
  second CLI, another machine — runs on the test account or from the test
  IP during a rung. Counters are server-side per account (N24), the token
  policy is IP-scoped (N33), and the spike sends the shipped registration's
  `client_id` and user-agent, so every violation lands on the shared
  registration (N10). Each rung's first probe must report 0 hits on its
  policy before the first GET is allowed to proceed.
- **One daemon per rung.** Every rung starts with `acq daemon stop`, and the
  daemon is started with `ACQ_IDLE_SHUTDOWN` long enough to outlive the
  rung, so rails configured by environment are actually in effect and the
  limiter history is continuous. The soak (rung 8) runs on a single
  long-lived daemon.
- **Post-violation rule.** After any tripwire trip: write the cause in the
  run ledger, wait at least **360 s** (the longest observed policy window
  plus the 60 s bucket), and only then `acq daemon reset-tripwire`. Never
  reset-and-retry to "see if it happens again."

## Blast-radius review (2026-08-22, read-only, at `c1be5c39`)

Every daemon HTTP path is one of four `ChokePoint` methods (`get_bearer`,
`get`, `head`, `post_form`), each holding a gate permit from before dispatch
through body completion. Worst-case send counts per daemon lifetime, and
what bounds them:

| Path | Trigger | Bound | Verdict |
| --- | --- | --- | --- |
| OAuth refresh POST | first auth-required job of a daemon lifetime (the access token is memory-only; only the refresh token survives in the keyring), then every ~10 h; also `acq auth check` | singleflight while a flight is open; paced by `token-request-limit` once learned (N33: 60/30 s); every success rotates the only refresh token | bounded; **R1** on failure |
| HEAD probe | first job on an `Unknown` route; the probe itself first needs a token (so a fresh daemon's first API job is POST + HEAD + GET) | one per route per lifetime; a failed probe (including a token failure) degrades the route for `PROBE_COOLDOWN` (60 s), after which the next job triggers one more | bounded: ≤ 1 HEAD / route / 60 s while jobs keep arriving |
| API GET | one per job attempt | limiter pacing from learned headers; 429 → requeue ≤ `MAX_429_RETRIES` (2) → 3 attempts max per job; 403/503 never retried; `Degraded` routes fail jobs without sending | bounded per job |
| OAuth code exchange POST | one per `acq auth` | user-driven | bounded |
| Daemon restart | CLI respawns at most once per invocation; idle exit after 60 s unless a policy window is live (≤ 300 s) | probe state and access token are in-memory, so every restart costs one POST plus one HEAD per used route | bounded per restart; a scripted loop of CLI calls across idle exits is the only stream shape (see rung 8 design) |
| Fan-out (`refresh --all`, `--deep`) | one `stash` child per tab / substash | data-bounded (tabs, substashes); each child is a normal job | bounded, but multiplies R2 |
| `profile` job | `acq submit profile` | fake-data kind that is **not** refused in real mode and calls `valid_access_token` → a real token POST | **R6** |

Risks found (none is a bug against the accepted N0–N6 contract; all are
gaps between "correct" and "safe to run unattended against GGG"):

- **R1 — dead refresh token is retried per flight.** A non-429 refresh
  failure (e.g. `400 invalid_grant` after revocation) ends the flight but
  leaves the refresh token in place, so the next auth-required job POSTs
  the same dead token. On a fresh daemon the probe fails first and degrades
  the route, so the shape is one POST per 60 s; the bad shape is a token
  that dies mid-fan-out after the access token expires, where every child
  job opens a flight. Paced by N33 once learned, so not a violation, but
  pointless traffic on a Cloudflare-fronted endpoint. (`daemon.rs`
  `finish_refresh`; `auth.rs` `token_request` collapses status, body, and
  JSON failures into one string.)
- **R2 — no global violation budget.** `Limiter::violations` is counted and
  never consulted. With a mis-modeled policy, a 50-child fan-out can burn
  up to 150 violations (3 attempts each), each behind a hold but each a
  counted violation against N10's unknown revocation threshold (Q8). One
  bad assumption should cost one violation, not one per job.
- **R3 — no HTTP timeout.** The `reqwest::Client` has neither connect nor
  request timeout. A hung send holds its gate permit and scheduling key
  indefinitely: the daemon stalls (safe direction) and nothing surfaces
  why. Note also that `ChokePoint::observe` records nothing on a transport
  error, so a request that times out *after* the server counted it leaves
  no history hit and the next send is under-paced.
- **R4 — no durable send record.** `ChokePoint::sends` is a 100-entry
  in-memory ring; the daemon log is prose and the daemon exits via
  `process::exit`, so buffered output is lost on every idle exit. The
  baseline needs every real send with its headers, across restarts.
- **R5 — no hard ceiling.** Nothing caps total real sends per lifetime. Not
  needed for correctness; wanted as the backstop for the bug nobody
  imagined, during the ladder only.
- **R6 — `profile` reaches the real token endpoint.** See the table.
- **R7 — a keyring save failure after rotation is a warning, not a stop.**
  The rotated refresh token is then memory-only; the next idle exit
  silently logs the account out. (`daemon.rs` `install_tokens_locked`.)

Sound and unchanged: probe-before-send on every unknown route; 429 bound
and hold; Cloudflare-shape non-retry; gate permit lifetime; singleflight
refresh; mock `fetch`/`demo` refusing to map to a route in real mode; CLI
respawn bounded to once per invocation.

## L0 — live-test rails (package)

Status: `built` and reviewed. Build range `7be3e7a9..2aa83f4d`; the
independent read-only review (question: *can any change add a send or
delay a halt?*) returned `changes-requested` with 4 Medium and 8 Low
findings, recorded in the L0 review register below and fixed in
`d7149374`. No re-review has been run. No semantic change to gate, limiter, OAuth,
classification, retry, or dispatcher behavior under default settings; the
rails only refuse or record.

Rail lifetime, decided up front so a later reader can tell scaffolding from
design: rails 3, 4, and 7 are **permanent** (ordinary hygiene); rails 1, 2,
and 5 are **ladder-only** — kept in the code as opt-ins, off by default
after the baseline, because they deliberately make the daemon more fragile
than CONTEXT.md's accepted 429-recovery decision intends.

1. **Tripwire (R2).** Opt-in via `ACQ_TRIPWIRE=1` (the ladder sets it;
   never on in mock mode by default, since the mock and the existing suite
   produce 429s deliberately). Trips on the first counted violation — any
   landed 429 on any route, including HEAD and token — and on any 403/503.
   A HEAD 429 is a designed-recoverable case under frozen D4; tripping on it
   is a deliberate ladder-time tightening, not a contradiction. While
   tripped, every `ChokePoint` transport method fails fast before acquiring
   a permit; a job already requeued by the triggering 429 fails at its next
   attempt rather than waiting behind the hold; `acq daemon status` and
   `acq dash` show the trip and its cause. Because the gate admits two
   concurrent sends, one already-dispatched request may still land after
   the trip: a 2-send row in the ledger is not a rail failure. Reset is
   explicit: `acq daemon reset-tripwire`. Persisted in a file keyed by
   provider (mock and real never share it) and honoring `ACQ_SOCKET`, so a
   respawned daemon stays tripped.
2. **Dead-token stop (R1).** Opt-in with the tripwire. `token_request`
   returns the HTTP status alongside the error so the daemon can tell a
   rejected grant from a network blip. A **4xx other than 429** on a
   **`refresh_token` grant** (never the code exchange, never 5xx or
   transport errors) marks the session `refresh-failed`; later refreshes
   fail fast without sending until `acq auth` or logout. Generation-checked
   like `finish_refresh`, so a stale flight cannot disable a session that
   re-authenticated meanwhile. Persisted alongside the tripwire so the idle
   exit does not clear it. Log the response evidence once.
3. **HTTP timeouts (R3).** Permanent. `connect_timeout` 10 s and a 60 s
   request timeout covering headers and body on the one client (both
   constructors). A timeout classifies as transport failure (existing
   `Network` path), no retry. On transport failure for a route with an
   established policy, push one conservative history hit so the next send
   is paced as if the lost request was counted.
4. **Send journal (R4).** Permanent, path from `ACQ_JOURNAL` (default next
   to the daemon log; `acq daemon status` prints it). Append-only JSONL,
   one line per actual send, written and flushed per line from
   `record_completed`, which is extended to receive the header snapshot and
   `counted` flag it currently lacks: wall-clock time, daemon pid, method,
   route, URL path, status or transport error, every `X-Rate-Limit-*`
   header and `Retry-After`, and `counted`. Never the Authorization header,
   a token body, or any response body. Predicted-wait is not in L0; drift
   is computed from consecutive header states and timestamps.
5. **Send ceiling (R5).** Ladder-only. `ACQ_MAX_SENDS=<n>`: after `n` real
   sends (HEAD, GET, and POST alike) in a lifetime the daemon halts with
   reason `ceiling`. Per-lifetime, **not** persisted — a respawn starts a
   fresh count — so the soak is not ended by its own ceiling. `acq daemon
   status` reports the configured ceiling and the count so far.
6. **Refuse `profile` in real mode (R6).** Permanent. Same treatment as
   `fetch`.
7. **Keyring save failure stops refresh (R7).** Permanent. A failed save
   of a rotated refresh token is surfaced as an error in `daemon status`
   and the ladder's stop condition; the in-memory token is kept so the
   session survives until exit.
8. **`ACQ_IDLE_SHUTDOWN=<secs>`.** Permanent knob, default unchanged
   (60 s). The ladder sets it per rung.

Required tests: each rail has a deterministic test against the mock and the
existing fake-clock/localhost harness, with rails 1, 2, and 5 forced on;
the existing suite (including the N6 stress, which contains 429s) passes
unchanged with rails 1, 2, and 5 off; a tripped daemon sends nothing on a
queued job. Quality gates from `NETWORK-CLEANUP.md` stay green.

### L0 review register

| ID | Sev | Finding | Resolution |
| --- | --- | --- | --- |
| L0-R1 | Medium | Halt checked only on entry to the admission loops; a task parked in a hold sleep or in the gate sent after the trip | re-checked every iteration and after admission; admission sleeps sliced to ≤ 1 s; test `a_send_parked_in_the_gate_is_refused_after_a_trip` |
| L0-R2 | Medium | Persisted trip / refresh-failed mark enforced by a rails-off daemon (behavior change under default settings) | loaded only when the tripwire is armed; a rails-off daemon neither honors nor deletes the file; test `persisted_trip_is_ignored_without_the_tripwire` |
| L0-R3 | Medium | `finish_auth_flow` cleared the dead-token mark before its generation check, so a stale callback re-armed a dead token | clear moved after a current flow installs tokens |
| L0-R4 | Medium | Journal open failure silent while status advertised the path | error kept; status shows `NOT WRITTEN — …`; logged at startup; test `unopenable_journal_is_reported_not_silent` |
| L0-R5 | Low | Mark set after the flight closed: one-instruction window for a second dead-token POST | mark set before `owner.finish`, only while the flight is still current |
| L0-R6 | Low | Stale-flight suppression was a string compare | superseded by L0-R5's explicit flight-current check under the lock; no prose compare remains |
| L0-R7 | Low | Default journal shared by mock and real | keyed by provider |
| L0-R8 | Low | Env knobs failed open silently | truthy set accepted; misunderstood values logged as `RAILS CONFIG` errors |
| L0-R9 | Low | `ACQ_MAX_SENDS=0` allowed one send | `halted()` refuses at zero; test `zero_ceiling_refuses_before_the_first_send` |
| L0-R10 | Low | `reset-tripwire` against a stopped daemon left the persisted trip | the CLI clears the provider's state file directly and says so |
| L0-R11 | Low | Per-job refusals could evict the trip cause from the error ring | per-job refusal goes to the file log only |
| L0-R12 | Low | Persisted refresh-failed cause contained the token-endpoint body | persisted cause is `HTTP <status>` plus a fixed reason; the body stays in the log only |

Clean per the review: `note_lost_send` is never less conservative; the
dead-token mark cannot fire on the code exchange; the fast path precedes
flight open/join and cannot strand waiters; `process()`'s early return
mirrors the `Degraded` return; journal contents carry no secret; CLI and
dash additions send nothing; N0–N6 behavior with rails off is unchanged.

## Ladder

Each rung has a stop condition; stopping means reading the journal before
the next rung, not retrying. `ACQ_GGG=1 ACQ_TRIPWIRE=1` throughout; journal
on; ceiling set per rung; `acq daemon stop` first. Expected counts assume
a fresh daemon (one refresh POST before the first API job). Record every
rung in the run ledger.

| Rung | Command(s) | Expect (POST/HEAD/GET) | Ceiling | Stop if |
| --- | --- | --- | --- | --- |
| 1 | `acq auth`, `acq auth status` | 1/0/0 (code exchange); `token-request-limit` learned with N33's shape | 3 | token response headers differ from N33; any non-2xx; keyring save warning |
| 2 | `acq auth check` | 1/0/0 (refresh); rotated refresh token persisted | 3 | refresh token not rotated; headers differ from N33; keyring save warning |
| 3 | `acq characters` | 1/1/1; probe reports 0 hits; policy shape matches ground truth | 5 | probe degrades; parser rejects headers; probe shows hits > 0; any 429 |
| 4 | `acq stashes --league Standard` | 0–1/1/1 under `stash-list-request-limit` | 5 | as rung 3 |
| 5 | `acq stash <id>` on one small tab | 0–1/1/1 under `stash-request-limit` | 5 | as rung 3 |
| 6 | `acq refresh --tabs a,b,c` (3 tabs) | 0–1/0–1/4; observed hits match limiter prediction | 8 | predicted vs. observed state drift > 1 hit |
| 7 | `acq refresh --tabs …` (≈10 tabs) over several minutes | 0–1/0/11; pacing engages; zero 429 | 16 | any 429 |
| 8 | soak: one daemon with `ACQ_IDLE_SHUTDOWN` ≥ 1 day; `acq characters` every 10 min by cron for several days | 1 POST per ~10 h, 1 HEAD per route per daemon lifetime, 1 GET per run; stable headers | 200 per lifetime | any trip; more than one HEAD per route per day; any keyring warning |

Rung 8 is the baseline. Anything the rungs teach about GGG goes into
`docs/design/network-ground-truth.md` as numbered claims and is promoted
to master promptly; this file records only runs.

## Run ledger

One row per rung execution. Journal files are copied to
`spikes/rust-playground/runs/<date>-<rung>/` (gitignored) and cited here.

| Date | Rung | Daemon tip | Result | Sends (POST/HEAD/GET) | Violations | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-08-22 | 1 | `92e74f93` | pass | 1/0/0 | 0 | code exchange 200; `token-request-limit` Ip `60:30:30`, state `1:30:0` (N33 confirmed for the `authorization_code` grant); access token 36000 s; keyring ok; `runs/2026-08-22-r1/` |
| 2026-08-22 | 2 | `92e74f93` | pass | 1/0/0 | 0 | fresh daemon; `auth check` refresh 200; same N33 headers, state `1:30:0` (prior hit aged out at +30.4 s); keyring ok; rotation proven indirectly by rung 3's refresh from the stored token; `runs/2026-08-22-r2/` |
| 2026-08-22 | 3 | `92e74f93` | pass | 1/1/1 | 0 | fresh daemon; refresh from stored token 200 (proves rung 2's rotation persisted), token state `2:30:0` as predicted; HEAD `/character` 204 reported `0:10:0,0:300:0` (exclusive use confirmed; HEAD uncounted, N24); GET 200 state `1:10:0,1:300:0`; policy `2:10:60,5:300:300` matches ground truth; 200 ms end to end; `runs/2026-08-22-r3/` |
| 2026-08-22 | 4 | `92e74f93` | pass | 1/1/1 | 0 | fresh daemon; HEAD `/stash/Standard` 204 reported `0:15:0,0:60:0`; GET 200 state `1:15:0,1:60:0`; `stash-list-request-limit` `10:15:60,30:60:300` matches ground truth; 261 tabs; `runs/2026-08-22-r4/` |
| 2026-08-22 | 5 | `92e74f93` | pass | 1/1/1 | 0 | fresh daemon; tab `ffaab719d0` (45 items); HEAD reported `0:10:0,0:300:0`; GET state `1:10:0,1:300:0`; `stash-request-limit` `15:10:60,30:300:300` matches ground truth; `runs/2026-08-22-r5/` |

## Review history

- 2026-08-22: plan reviewed before build (owner, author, and one
  independent read-only agent). Corrections folded in: per-rung send
  counts include the refresh POST; R1's worst case restated; `profile`
  (R6) and keyring-save (R7) added; rails 1/2/5 made opt-in and the
  tripwire file keyed by provider; rail 2 given status-based and
  generation-checked semantics; rail 3's observe blind spot; rail 4's
  flush-per-line and no-secrets rules; `ACQ_MAX_SENDS` made per-lifetime;
  env knobs reported in `daemon status`; preconditions and the 360 s
  post-violation rule added; soak moved to one long-lived daemon.

## Next action

Owner decides whether the L0-R1…R12 fix commit gets a fix-only re-review
or goes straight to rung 1.
