# Live testing control

Control document for running the Rust daemon against the real GGG API:
the standing rule for first contact with anything new, the safety rails,
the closed ladder (history), and the run ledger. `CONTEXT.md` invariants
apply. Ground-truth facts learned live go to
`docs/design/network-ground-truth.md` as numbered claims (authored
master-side, cherry-picked here); this file records only runs.

The ladder's goal — confidence that the daemon **halts rather than
floods** — was met on 2026-08-27 (status at the end). The ladder was the
right ceremony for an unproven limiter; with the limiter proven, its
paperwork half (a written hypothesis before any live run) is retired
(owner decision, 2026-08-30) and what survives is the part with teeth: the
rails, which are code, and the rule below.

## Standing rule: first contact (2026-08-30)

Replaces the preconditions and the "new hypothesis first" requirement.

- **First live call on a new endpoint runs with the rails on**:
  `ACQ_GGG=1 ACQ_TRIPWIRE=1 ACQ_MAX_SENDS=3`, from a terminal, on a fresh
  daemon (`acq daemon stop` first). Three, not one: a fresh daemon sends a
  token POST, then the HEAD probe, then the GET, and the ceiling counts
  every method (`rails.rs`, "ceiling is per lifetime and counts every
  method"). Two if the daemon already holds a valid access token.
- **Read the journal before anything else.** The probe line must report
  0 hits on its policy; hits > 0 means something else is using this
  account (the C++ app, another CLI, another machine — counters are
  per account, N24/rung 11) — stop and find it. The probe is the
  hypothesis mechanism: it learns the policy before the first counted
  send, and a probe that fails or comes back without rules closes the
  endpoint for 60 s (N20 path).
- **Record the observed policy** (name, windows, the state after the
  GET) as one row in the run ledger and as a ground-truth claim
  master-side. That row is what lets `ratelimit.rs`'s test table cite it.
- **Post-violation: wait 360 s, then `reset-tripwire`; never
  reset-and-retry.** After any tripwire trip write the cause in the
  ledger, wait at least 360 s (the longest observed policy window plus
  the 60 s bucket), and only then `acq daemon reset-tripwire`. A trip
  is evidence to read, not a retry prompt. This is a fact about GGG, not
  paperwork, and it outlives the ladder.
- **A failed probe leaves only headers.** A HEAD has no body, so a
  probe's non-2xx is classified from its response headers, which the
  journal (`headers`), the daemon log, and the trip cause all carry
  (since the `/profile` 403 below). Routes known not to accept HEAD
  skip the probe (`route_probes` in `daemon.rs`) and are taught by
  their first GET instead.
- **Verify the binary, not the checkout.** `acq --version` must equal
  `git rev-parse --short=12 HEAD` with no `-dirty`; never rebuild
  `target/debug/acq` under a live daemon without `acq daemon stop` first
  (rung 8 ran 34 h on a binary that predated the fix it was restarted to
  pick up).

Endpoints real but unsampled as of 2026-08-30: `GET /profile`,
`GET /character/{name}`, `GET /league` (job kinds `profile`, `character`,
`leagues`, added in `fa74c5ef`). Each gets first contact under this rule
— step (7) of the multi-account build order in `CONTEXT.md`.

History moved out of this file on 2026-08-24 and lives in git: the
blast-radius review (risks R1–R8, at `c1be5c39`), the L0 rails build and
its review register (L0-R1–R13), and the review history. The file at
`9fa99459` holds the full text. All of R1–R8 are resolved: R1 by the
dead-grant decision (`CONTEXT.md`), R2–R7 by the rails below, R8 fixed in
`529bdd92` and seen live once (rung 8).

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
6. **The mock-only kinds (`whoami`, `fetch`, `sleep`) are refused or
   unroutable in real mode** (permanent). `profile` is real since
   `fa74c5ef` (2026-08-29), as are `character` and `leagues`; all three
   are unsampled — see the standing rule. (This rail's text said
   `profile` was refused until 2026-08-30; that was stale from the
   rename of the old mock exerciser to `whoami`.)
7. **Keyring save failure is surfaced** in `daemon status` and is a stop
   condition (permanent); the in-memory token is kept until exit.
8. **`ACQ_IDLE_SHUTDOWN=<secs>`** (permanent knob), set per rung.

Each rail has a deterministic test against the mock with rails 1 and 5
forced on; the suite passes unchanged with them off; quality gates from
`NETWORK-CLEANUP.md` stay green.

## Ladder (closed 2026-08-27; kept as history)

The preconditions it ran under (exclusive use; one daemon per rung with
`ACQ_IDLE_SHUTDOWN` outliving it; binary provenance; the 360 s
post-violation rule; ceilings derived from the rung's own counts, never
another rung's) are in git at `26850097` and earlier; the ones that are
facts about GGG rather than ceremony live on in the standing rule above.
Each rung had a stop condition; stopping meant reading the journal, not
retrying. `ACQ_GGG=1 ACQ_TRIPWIRE=1` throughout; journal on; ceiling per
rung; fresh daemon (one token POST before the first API job).

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

Rung 8 mechanics: the daemon is started **from a terminal** by a person
(a daemon spawned by cron has no keychain access on macOS and no session);
`tools/soak-run.sh` is the cron body (sets the same rails env and
`ACQ_NO_SPAWN=1` so it can only talk to that daemon, derives the ceiling as
`SOAK_DAYS` × 144 GETs + token POSTs + probes, runs one `acq characters`
from a **frozen copy of the binary** at `runs/soak/acq` so the tree can be
rebuilt while the daemon lives, appends one line to `runs/soak/runs.log`); `tools/soak-check.sh <start-ts>`
evaluates the stop conditions from the journal, the run log, and daemon
status: it refuses a manual-clock journal and any lifetime in the window
not built from the binary on disk, and counts HEADs per `(pid, route)`.
Laptop sleep is welcome, not avoided — cron skips the sleeping minutes and
the wake is what exercises R8 (an expired token refreshed before the
next GET, never a 401).

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

| 2026-08-25 | 10 (rerun) | `3b5e0282` | **pass** | 1/2/323 = 326 | 0 | pid 96766, 03:36–04:37 UTC, 61 min. Probes 0 hits; 322 tabs; the 15-per-10 s / 30-per-300 s pattern ten times, every hold ~343 s and every first send after a hold answered `1:10:0,1:300:0`; windows never exceeded 15/30; zero non-2xx; no keyring warning. Snapshot written: 322 tabs, 18 072 items, 0 errors. Tab `7b05e6f78d` (the earlier 503, "Beasts, Red (Remove-only)", 37 items) was again the first send after a hold — send #245, same position — and answered 200: the 503 was transient, not tab-specific, and one clean after-hold sample says nothing yet about stale connections. Second-run "no changes" check not yet run. `runs/2026-08-24-r10b/` |

| 2026-08-25 | 10 (second run) | `5d792d0a` | **pass** (wire); diff false positive fixed client-side | 1/2/323 = 326 | 0 | pid 6111, 11:19–12:20 UTC, 61 min; fresh daemon, probes 0 hits; identical to the rerun on the wire: ten ~343 s holds, `1:10:0,1:300:0` after each, max 15/30, zero non-2xx, no keyring warning. Snapshot 322 tabs / 18 072 items / 0 errors; no tab or item added, removed, or moved. Reported **10 items changed** — all `veiledMods`, whose placeholder ids GGG re-randomizes per fetch (`Prefix06` → `Prefix01`); not stash changes. The diff now ignores that field (`pull.rs`, `VOLATILE_ITEM_FIELDS`); new ground-truth observation for master-side. `runs/2026-08-25-r10c/` |

| 2026-08-25 | 8 re-soak (first tick) | `fe249193` | **no sends; stopped before start** | 0/0/0 | 0 | cron's first tick at 14:30Z lazy-spawned the daemon (pid 13589) under cron's context; macOS Keychain refused (`User interaction is not allowed`), so it came up with no session and the run failed "not logged in" before any send. Rail 7 surfaced it in `daemon status`. Rung 8's daemon had been spawned from a terminal and outlived the whole run, so this never showed. Fix: `ACQ_NO_SPAWN=1` in `soak-run.sh` (cron can only talk to a daemon a person started); the stored refresh token was never read and is intact. |

| 2026-08-25 → 27 | 8 re-soak | `a7873d21` (frozen `runs/soak/acq`) | **pass** — 45.3 h, 2026-08-25T14:33:40Z → 2026-08-27T11:53Z | 4/2/133 = 139 (+1 transport failure) | 0 | Two lifetimes (pid 14571 from a terminal; pid 43863 after a deliberate `daemon stop` at 21:52Z day 2), cron every 10 min with `ACQ_NO_SPAWN=1`, 132 runs, ceiling 304. Every GET `1:10:0,1:300:0`; exactly one HEAD per lifetime; four token POSTs, each immediately before the GET that needed it. **R8 seen fixed live twice**: token expiry inside a 15 min sleep (10:40Z day 2, refresh at 10:50Z) and inside a **10 h sleep** (07:52Z day 3, refresh at 11:40:51Z, GET 170 ms behind it) — zero 401s. One `error sending request` at 02:59Z day 2: cron fired 9 min late out of a sleep, network not up; paced as counted, next run clean. No trip, no keyring warning. Fewer sends than the cadence × duration estimate because the closed laptop slept most of both days — which is what produced the evidence. `runs/2026-08-25-r8b/` (journal, daemon log, run log, `soak-check.txt`, `pmset` sleep/wake log) |

| 2026-08-30 | 11 | `227fca80` | **pass** for H1, H2 (H0 missed, H3 not run) | A 2/1/2, B 1/1/2 = 9 | 0 | pids 43950 (A, account 1) and 44048 (B, account 2), both `ACQ_NO_KEYRING=1`, 03:03–03:07 UTC. **H1 confirmed**: B's HEAD `/character` at 03:05:12.8, **4.1 s after A's counted GET**, reported `0:10:0,0:300:0`, and B's GET was answered `1:10:0,1:300:0` — `Account` rules count per account, not per IP or client. **H2 confirmed**: both daemons' GETs went out **28 ms apart** (03:06:03.835 / .863), each answered `1:10:0,2:300:0`, `wait_ms` 0, no HEAD, neither saw the other's hit. H0 unsampled: the two code exchanges were 31.2 s apart, so A's token hit had just aged out (`1:30:0` on both); consistent with N33, proves nothing new. H3 not run: daemon A re-logged in as account 2 at 03:06:43 (`logged in as` the second account, no HEAD, no re-probe) but was stopped 13 s later before `acq characters` — the carry-over stays a mock-only observation. Zero non-2xx, no trip. `runs/2026-08-30-r11/` (journals + daemon logs) |

| 2026-08-30 | first contact: `profile` | `fdb2d20f`-dirty | **halted by tripwire at send 2** | 1/1/0 | 0 × 429; **1 × 403 on HEAD** | Fresh daemon, second account, `acq profile` 27 s after login: the first-use **HEAD `/profile` was answered 403**; the GET never went out. No headers or body recorded — a HEAD has no body, and the daemon did not yet log a probe's response headers; the journal was not written because its directory did not exist (both fixed the same day: header snapshot on every non-2xx, journal directory created on demand). Two candidate causes, undecided: `/profile` does not accept HEAD (the N24 allowance has only been seen on `/character` and `/stash`), or the `account:profile` scope was not granted. Not a Cloudflare burst (2 sends). `/profile` now skips the probe; the next sample is one GET with ceiling 1 (token still valid). Binary provenance rule was not met (`-dirty`); does not affect the finding. |

### Re-soak postmortem (2026-08-27)

What the re-soak had to show, it showed: the R8 fix live (twice, once
across a ten-hour sleep), the HEAD condition able to fail and not failing
across a restart, and a `wait_ms` baseline of zero on a route that never
saturates. Three things it taught that were not on the list:

- **A daemon spawned from cron has no keychain.** macOS refuses secure
  storage to non-interactive callers; the first tick came up with no
  session and failed before any send — rail 7 caught it. Now
  `ACQ_NO_SPAWN=1` (README): cron only talks to a daemon a person started.
- **"Sleep" is not one thing.** On AC, Power Nap dark-wakes the closed
  laptop every 15–60 min and cron runs during the dark wakes; on battery
  it sleeps for hours. Both produced expiry-spanning samples, but only
  the battery night produced a long one. `pmset -g log` is part of the
  evidence for any sleep claim.
- **The first request after a wake can fail in transport** (network not
  up). Rail 3 paces it as counted; a product consumer sees one failed
  job. Same gap as "refetch only the failed set" (`CONTEXT.md`).

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

## Rung 11 — two accounts, one machine (hypothesis, 2026-08-29)

Question: what does the rate limiter do when two accounts are logged in
and sending at the same time? Everything the code implies is untested
against GGG, so this rung asks the servers rather than reasoning from the
ground-truth table. What the code does is known and is the thing under
test: the limiter is keyed by policy name only (`ratelimit.rs`), one
daemon holds one session, login does not reset counters or re-probe, and
each daemon has its own limiter. Seen on the mock 2026-08-29: after an
account switch the first request is paced on the previous account's
history with no new probe.

Hypotheses (H1 decides whether H2–H3 run):

- **H1 — `Account` rules count per account, not per IP or client.** A
  HEAD probe on account B, seconds after account A's counted GET on the
  same policy, reports `0` hits. If it reports A's hit, counters are
  shared across accounts on this machine and the single-limiter design is
  right by accident; stop there and record it.
- **H2 — two daemons on two accounts sending simultaneously do not see
  each other** in `character-list-request-limit` headers (each GET is
  answered `1:10:0`), and neither is held.
- **H3 — an account switch on one daemon carries the old account's
  counters** (predicted from the code, never seen live): after A's hits,
  `acq auth` as B on daemon A, then `acq characters` — no HEAD, `wait_ms`
  computed from A's history, and the response state is B's real count.
  A conservative gap, not a violation; the fix shape is "forget
  `Account`-scoped policies on session change".
- **H0 (free)** — two code exchanges within 30 s from one IP show
  `token-request-limit` state `2:30:0` on the second (N33 says Ip-scoped;
  this is the first cross-account sample).

Preconditions and an explicit exception: the ladder's exclusive-use rule
forbids two instances from one IP; this rung *is* two instances from one
IP, by owner decision, on `character-list-request-limit` only (2 per
10 s, 5 per 300 s — low enough that the shared-counter case cannot 429
within the plan). Both daemons run `ACQ_NO_KEYRING=1`, so the stored
refresh token of the real account is never read or overwritten. Every
`acq` call goes through `tools/acq-as.sh A|B …`, which sets the rails
(`ACQ_GGG=1 ACQ_TRIPWIRE=1`, ceiling 8, idle 3600 s), one socket per
label (own limiter, own tripwire file, own store) and one journal per
label under `runs/<date>-r11/`. Binary provenance as always; both
daemons on the same tip. Account B needs its own browser login
(private window): `acq auth --no-browser` prints the URL.

| Step | Where | Command | Expect | Stop if |
| --- | --- | --- | --- | --- |
| 0 | A, then B within 30 s | `tools/acq-as.sh A auth --no-browser`, approve as account A; same for B as account B | POST 200 each; B's token state `2:30:0` (H0) | any non-2xx |
| 1 | A | `tools/acq-as.sh A characters` | HEAD `0:10:0,0:300:0` (both accounts quiet), GET `1:10:0,1:300:0` | probe hits > 0; any non-2xx |
| 2 | B, within 10 s of step 1 | `tools/acq-as.sh B characters` | **H1**: HEAD `0:10:0,0:300:0` → per account, continue; HEAD `1:10:0,1:300:0` → shared, **stop and record** | any non-2xx |
| 3 | both, ≥ 15 s after step 2 | `tools/acq-as.sh A characters & tools/acq-as.sh B characters & wait` | **H2**: both GET 200, each `1:10:0,2:300:0`, `wait_ms` 0 on both, no HEAD | either held; either state shows the other's hit; any non-2xx |
| 4 | A | `tools/acq-as.sh A auth --no-browser`, approve as **account B**; then immediately `tools/acq-as.sh A characters` | **H3**: POST 200; no HEAD; GET state is B's count (`…,3:300:0`); `wait_ms` > 0 only if A's 10 s window was still open | a HEAD is sent (would mean login re-probes — record, not a fault); any non-2xx |
| 5 | both | `tools/acq-as.sh A daemon stop; tools/acq-as.sh B daemon stop` | | |

Expected totals: A 2 POST / 1 HEAD / 3 GET, B 1 / 1 / 2; ceiling 8 each.
Under the shared-counter branch of H1 the plan ends after step 2 with 2
GETs in a 5-per-300 s window. Result rows go in the run ledger above;
anything learned about GGG goes to ground truth master-side.

Run 2026-08-30 (ledger row above): **H1 and H2 hold** — per-account
counters, no cross-account interference between two daemons on one IP.
H3 (the account-switch carry-over on one daemon) was not sampled; it is
predicted from the code and seen on the mock, is conservative
(over-waiting only), and is not worth a further live run on its own.
H0 was missed by one second and is already covered by N33.

## Status: ladder closed (2026-08-27)

Every rung has passed except rung 9, deferred on purpose (each attempt is
a counted violation; rung 10's twenty holds bound the bucket well enough).
Across the ladder: ~1,450 live sends, **zero 429s**, one transient origin
503 (N35) handled without a retry, R8 seen fixed live, and the rails
proven on three real incidents (the 503, a ceiling derived from a stale
count, a keyring-blind spawn). The goal this document set — the daemon
**halts rather than floods** — is met with evidence, and the GGG-side
boundary is mapped to diminishing returns. Rung 11 (2026-08-30) was the
one addition after closing, run as a written hypothesis because it asked a
question about GGG (per-account counting) rather than about a new
endpoint. No further rungs are planned. Live contact from here follows
the standing rule at the top: rails on, ceiling 3, read the journal,
record the policy — a ledger row, not a hypothesis document. A run that
asks a genuinely new question of GGG (like rung 11) is still worth
writing down first; that is judgment, not a rule.

The frontier is the frontend boundary and the multi-account build
(`CONTEXT.md`).
