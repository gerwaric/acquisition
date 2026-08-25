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
| 10 | `acq pull --league Standard` (the first real consumer; no `--deep`) | 0–1/2/1+N with N = tabs listed (261 at rung 4); `stash-request-limit` is 30 per 300 s, so ~9 holds of up to 5 min, wall clock near 45 min; zero 429; snapshot written; a second run on the same daemon reports no changes with 1+N GETs and no new HEADs (probes are per daemon lifetime) | N + 10 | any 429; any reported window state with hits > max; tabs the list reported missing from the snapshot with no error recorded |

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

## Next action

Rung 8 is stopped, not passed, and the R8 fix has never executed against
GGG. Two live runs are pending, in either order:

- **Re-soak** on the verified binary, ceiling from cadence × duration, HEAD
  condition per `(pid, route)`; collects the first live `wait_ms` baseline.
- **Rung 10, `acq pull`** — the first live run that exercises the 300 s
  window repeatedly; ceiling and duration are derived in the ladder row.
