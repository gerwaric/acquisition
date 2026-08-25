# Live testing control

Control document for running the Rust daemon against the real GGG API:
preconditions, the safety rails, the staged ladder, the run ledger, and
the next action. `CONTEXT.md` invariants apply. Ground-truth facts learned
live go to `docs/design/network-ground-truth.md` as numbered claims
(authored master-side, cherry-picked here); this file records only runs.

Goal: confidence that the daemon **halts rather than floods** — no HEAD
streams, no repeated rate-limit violations — so a baseline can be collected
before further development or refactoring. Not a correctness proof.

History moved out of this file on 2026-08-24 and lives in git: the
blast-radius review (risks R1–R8, at `c1be5c39`), the L0 rails build and
its review register (L0-R1–R13), and the review history. The file at
`9fa99459` holds the full text. All of R1–R8 are resolved: R1 by the
dead-grant decision (`CONTEXT.md`), R2–R7 by the rails below, R8 fixed in
`529bdd92` and seen live once (rung 8).

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
  limiter history is continuous.
- **Verify the binary, not the checkout.** `acq --version` must equal
  `git rev-parse --short=12 HEAD` with no `-dirty`; the daemon's first log
  line and the journal's `open` header say the same. Never rebuild
  `target/debug/acq` under a live daemon without `acq daemon stop` first —
  the version handshake would not notice. (Rung 8 ran 34 h on a binary
  that predated the fix it was restarted to pick up.)
- **Post-violation rule.** After any tripwire trip: write the cause in the
  run ledger, wait at least **360 s** (the longest observed policy window
  plus the 60 s bucket), and only then `acq daemon reset-tripwire`. Never
  reset-and-retry to "see if it happens again."
- **Ceilings are derived, not guessed:** cadence × intended duration for a
  soak; listed tabs + probes + refresh for a pull.

## Rails

Knobs and defaults are in the README ("Live-test rails"); `acq daemon
status` prints their state and `acq dash` shows a halt in red. What each
one is, and whether it outlives the ladder:

1. **Tripwire** (`ACQ_TRIPWIRE=1`, ladder-only). The first landed 429 on
   any route, HEAD and token included, or any 401/403/503, halts every
   later send until `acq daemon reset-tripwire`; persisted per provider
   across restarts. Tripping on a HEAD 429 is a deliberate ladder-time
   tightening of the accepted 429-recovery decision, which is why the rail
   is off by default. The gate admits two concurrent sends, so one
   already-dispatched request may still land after a trip: a 2-send row in
   the ledger is not a rail failure.
2. **Dead-grant stop** — product behavior since 2026-08-24, not a knob: a
   `refresh_token` grant rejected with a 4xx other than 429 is never
   re-sent until `acq auth` or logout.
3. **HTTP timeouts** (permanent): 10 s connect, 60 s request; a send lost
   in transport is paced as if the server counted it.
4. **Send journal** (`ACQ_JOURNAL`, permanent): one JSON line per actual
   send, never a token or body; the contract surface (`TESTING-NOTES.md`).
5. **Send ceiling** (`ACQ_MAX_SENDS=<n>`, ladder-only): halt after `n`
   sends this lifetime; not persisted, so a soak's respawn starts fresh.
6. **`profile` and the mock-only kinds are refused in real mode**
   (permanent).
7. **Keyring save failure is surfaced** in `daemon status` and is a stop
   condition (permanent); the in-memory token is kept until exit.
8. **`ACQ_IDLE_SHUTDOWN=<secs>`** (permanent knob), set per rung.

Each rail has a deterministic test against the mock with rails 1 and 5
forced on; the suite passes unchanged with them off; quality gates from
`NETWORK-CLEANUP.md` stay green.

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
| 7b | `acq refresh --tabs …` (18 tabs, > 15-per-10 s) | 1/2/19; the limiter holds before the 16th child; zero 429 | 24 | any 429; no hold observed |
| 8 | soak: one daemon with `ACQ_IDLE_SHUTDOWN` ≥ 1 day; `acq characters` every 10 min by cron | 1 POST per ~10 h, 1 HEAD per `(pid, route)`, 1 GET per run; stable headers | cadence × duration | any trip; more than one HEAD per `(pid, route)`; any keyring warning |
| 9 | *deferred* — timing-bucket measurement (owner decision 2026-08-23): each early guess is a counted 429 against N10's unknown threshold (Q8); if ever run, `character-list-request-limit` (2 per 10 s), a handful of violations total, the 360 s rule between attempts. The zero-violation alternative is asking GGG (N14). Rung 7b's one data point bounds the initial bucket at ≤ 5 s. | | | |
| 10 | `acq pull --league Standard` (the first real consumer; no `--deep`) | 0–1/2/1+N with N = tabs listed **including folder children** (322 on 2026-08-24; rung 4's 261 was the flat count); `stash-request-limit` is 30 per 300 s, so ~9 holds of up to 5 min, wall clock near 45 min; zero 429; snapshot written; a second run on the same daemon reports no changes with 1+N GETs and no new HEADs (probes are per daemon lifetime) | N + 10, from a `pull`'s own listed count, not an earlier rung's | any 429; any reported window state with hits > max; tabs the list reported missing from the snapshot with no error recorded |

Rung 8 mechanics: `tools/soak-run.sh` is the cron body (sets the rails
env itself so a respawned daemon keeps them, runs one `acq characters`,
appends one line to `runs/soak/runs.log`); `tools/soak-check.sh <start-ts>`
evaluates the stop conditions from the journal, the run log, and daemon
status, and refuses a journal whose `build` it cannot trust.

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
| 2026-08-22 | 6 | `92e74f93` | pass | 1/2/4 | 0 | fresh daemon, 3 tabs; both routes probed; stash HEAD reported `0:10:0,1:300:0` — rung 5's hit from 15 s earlier on a previous daemon, learned before the first send; final `3:10:0,4:300:0` = prior + 3; drift 0; 600 ms end to end; `runs/2026-08-22-r6/` |
| 2026-08-22 | 7 | `92e74f93` | pass (pacing not exercised) | 1/2/11 | 0 | fresh daemon, 10 tabs in 1.4 s; probe taught `4:300`; final `10:10:0,14:300:0` = prior + 10, drift 0; zero 429. 10 < 15-per-10 s, so the limiter never had to hold — rungs 1–7 prove reading, not waiting; `runs/2026-08-22-r7/` |
| 2026-08-22 | 7b | `92e74f93` | pass (pacing engaged) | 1/2/19 | 0 | fresh daemon, 18 tabs; 15 children in 1.4 s filled `stash-request-limit`'s 10 s window (`15:10:0`); the limiter held **14.75 s** (period + 5 s bucket) before the 16th, which the server answered `1:10:0` — window fully cleared; remaining 3 at full speed; final `3:10:0,18:300:0`; zero 429; ceiling 24; `runs/2026-08-22-r7b/` |
| 2026-08-22 | 8 (first start) | `92e74f93` | stopped after 3 runs | 1/1/1 + 2 GET | 0 | pid 14352, 00:26–01:30 UTC; stopped to pick up the R8 sleep fix; all runs success |
| 2026-08-23 | 8 | `92e74f93` — **not** `529bdd92` | **stopped, not a pass**: ceiling 200/200 at 2026-08-24T10:10Z, 34.1 h in | 4/1/195 = 200 | 0 × 429; **3 × 401** | one daemon (pid 17066, `ACQ_IDLE_SHUTDOWN=604800`), cron every 10 min, 210 runs. Steady state clean: zero 429, `1:10:0,1:300:0` on every GET, 1 HEAD for the lifetime, 4 token POSTs (one per ~10 h; `expires_in` 36000 each), no keyring warnings. Postmortem below. Evaluate with `tools/soak-check.sh 2026-08-23T01:30:14Z`; `runs/2026-08-23-r8/` |

| 2026-08-24 | 10 (attempt 1) | `18b68aa2` | aborted by owner | 1/2/61 | 0 | pid 87261; both probes 0 hits; list returned **322 tabs** (folder children included) against a ceiling of 271 derived from rung 4's 261, which would have halted ~55 tabs short — stopped after the second 10 s window to restart with the derived ceiling; `runs/2026-08-24-r10/` (one journal, filter by pid) |
| 2026-08-24 | 10 (attempt 2) | `18b68aa2` | aborted by owner (interrupted shell) | 1/2/1 | 0 | pid 88037, ceiling 332; the stash probe reported `0:10:0,30:300:0` — attempt 1's hits, learned before any send; stopped before its first GET |
| 2026-08-24 | 10 | `18b68aa2` | **halted by tripwire, not a pass**: 503 at send 245/332 | 1/2/242 (241 GET 200 + 1 GET 503) | 0 × 429; **1 × 503** | pid 88389, 02:05–02:54 UTC. Probes 0 hits; 15 GETs per 10 s window, 14 s hold, 15 more, then a **~343 s hold** (300 s + 60 s bucket − elapsed) — seven times; the first GET after every hold was answered `1:10:0,1:300:0`, prediction exact, windows never exceeded 15/30. The first send after the 8th hold (`GET /stash/Standard/7b05e6f78d`) got **503 with no rate headers** and an **openresty** "Service Temporarily Unavailable" HTML body — GGG's origin, not a Cloudflare 1015/403 shape (N3, N28). Tripwire halted; 0 sends after it; the remaining 82 children failed as "halted by rails" without sending. Not retried (invariant 3). `pull` wrote no snapshot. Postmortem below. |

### Rung 8 postmortem (2026-08-24), in three lines

The daemon was restarted 7 s after `529bdd92` was committed but `cargo
build` was never run, so the soak ran on `92e74f93` — every rail correct
and blind to it; hence the binary-provenance precondition above and the
build stamp in `--version`, the log, and the journal. R8 was then observed
live on that unfixed code: ~2029 s of laptop sleep froze the monotonic
expiry clock, so three GETs went out with an expired token (401 ×3) before
the refresh fired late — the first live sighting of a hazard found by
reading. Two limits of the run itself: the "one HEAD per route" condition
could not fail without restarts, and the 200-send ceiling was 33 h against
a "several days" intent — both folded into the preconditions and the rung
8 row.

### Rung 10 postmortem (2026-08-24)

The limiter's behaviour across eight consecutive 300 s windows is the
best evidence this ladder has produced: the hold is the full remainder
plus the 60 s bucket, and the server's counters were at zero after every
one. The stop was a server-side 503 from GGG's origin (openresty page, no
`X-Rate-Limit-*`, no `cf-ray` shape recorded) on the first send after a
5-minute idle; the account had 0 hits in both windows at that moment, so
it is not a violation and not evidence about our pacing. It is a new
ground-truth observation — a transient 503 shape distinct from N3/N28 —
to be authored master-side. The rails did exactly what they are for:
one send landed, nothing followed. Three things it exposed that are ours:

- **A ceiling derived from another rung's count is a guess.** The pull
  lists more tabs than `acq stashes` reported at rung 4 (folder children).
  Derive from the pull's own listing; rung 10's row now says so.
- **A tripwire halt fails every queued job with no send** — 82 jobs marked
  `failed` without ever reaching GGG, so a rerun refetches all 322 tabs
  (~1 h of holds). Whether a halt should leave jobs *waiting* rather than
  failed is a design question for the owner (`CONTEXT.md`, frontend
  findings).
- **A pull that fetched 240 of 322 tabs wrote nothing.** Partial results
  are discarded on any child failure. Recorded as a frontend finding.

## Next action

Rung 10 halted on a server-side 503 with the account at zero hits (see
postmortem); the 360 s post-violation wait has elapsed. Pending, owner's
order:

- **Rung 10 rerun** on a fresh daemon: `acq daemon stop`, then
  `acq daemon reset-tripwire` (the trip is persisted per provider and a
  new daemon honours it), ceiling 332 from the listed count. The 503's tab (`7b05e6f78d`) is the one to watch: if it 503s
  again the fault is tab-specific, otherwise it was transient.
- **Re-soak** on the verified binary, ceiling from cadence × duration, HEAD
  condition per `(pid, route)`; collects the first live `wait_ms` baseline.
- **Ground truth:** author the openresty-503 observation as a new claim
  master-side (`runs/2026-08-24-r10/job-243-503.json` holds the body).
