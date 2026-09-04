# Live testing control

Control document for running the Rust daemon against the real GGG API:
the standing rule for first contact with anything new, the safety rails,
the closed ladder (history), and the run ledger. `CONTEXT.md` invariants
apply. Ground-truth facts learned live go to
`docs/design/network-ground-truth.md` as numbered claims (authored
master-side, cherry-picked here); this file records only runs.
Closed rung sections were cut to their records on 2026-09-02; the full
text (preparation tables, prompts, agent observations) is at `d660d1f5`,
and what the runs taught is `REFRESH-SLICE.md`.

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

Every job kind has had first contact as of 2026-08-30 (run ledger:
`profile`, `leagues`, `character`; `leagues` was routed to `/league` until
that day), and the two poe2 character routes had theirs on 2026-09-02
(N41: poe2-suffixed policy names, free HEAD). A new kind gets the same treatment under this rule. What the
samples found that headers could not teach (`/profile`: no rate headers,
HEAD 403; `/account/leagues`: HEAD counted) is carried as declared route
knowledge (`CONTEXT.md` decisions; ground truth N38/N39, Q12).

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
   across restarts. Since 2026-08-30 a halt leaves queued jobs *waiting*
   (on disk, in `daemon.db`) rather than failing them: the halted daemon
   idles out and its successor holds the queue until the reset — so
   `acq jobs` before `reset-tripwire`, and `acq cancel` what should not go
   out. Tripping on a HEAD 429 is a deliberate ladder-time
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
   **The queue is persisted, the ceiling is not**: jobs halted by the
   ceiling resume under the next daemon's fresh ceiling. Before respawning
   after a ceiling halt, `acq jobs` and cancel what the run did not mean to
   send; a first-contact ceiling of 3 bounds one lifetime, not the queue.
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

| 2026-08-25 | 10 (second run) | `5d792d0a` | **pass** (wire); diff false positive fixed client-side | 1/2/323 = 326 | 0 | pid 6111, 11:19–12:20 UTC, 61 min; fresh daemon, probes 0 hits; identical to the rerun on the wire: ten ~343 s holds, `1:10:0,1:300:0` after each, max 15/30, zero non-2xx, no keyring warning. Snapshot 322 tabs / 18 072 items / 0 errors; no tab or item added, removed, or moved. Reported **10 items changed** — all `veiledMods`, whose placeholder ids GGG re-randomizes per fetch (`Prefix06` → `Prefix01`); not stash changes. The diff now ignores that field (the retired `pull` command, `VOLATILE_ITEM_FIELDS`); new ground-truth observation for master-side. `runs/2026-08-25-r10c/` |

| 2026-08-25 | 8 re-soak (first tick) | `fe249193` | **no sends; stopped before start** | 0/0/0 | 0 | cron's first tick at 14:30Z lazy-spawned the daemon (pid 13589) under cron's context; macOS Keychain refused (`User interaction is not allowed`), so it came up with no session and the run failed "not logged in" before any send. Rail 7 surfaced it in `daemon status`. Rung 8's daemon had been spawned from a terminal and outlived the whole run, so this never showed. Fix: `ACQ_NO_SPAWN=1` in `soak-run.sh` (cron can only talk to a daemon a person started); the stored refresh token was never read and is intact. |

| 2026-08-25 → 27 | 8 re-soak | `a7873d21` (frozen `runs/soak/acq`) | **pass** — 45.3 h, 2026-08-25T14:33:40Z → 2026-08-27T11:53Z | 4/2/133 = 139 (+1 transport failure) | 0 | Two lifetimes (pid 14571 from a terminal; pid 43863 after a deliberate `daemon stop` at 21:52Z day 2), cron every 10 min with `ACQ_NO_SPAWN=1`, 132 runs, ceiling 304. Every GET `1:10:0,1:300:0`; exactly one HEAD per lifetime; four token POSTs, each immediately before the GET that needed it. **R8 seen fixed live twice**: token expiry inside a 15 min sleep (10:40Z day 2, refresh at 10:50Z) and inside a **10 h sleep** (07:52Z day 3, refresh at 11:40:51Z, GET 170 ms behind it) — zero 401s. One `error sending request` at 02:59Z day 2: cron fired 9 min late out of a sleep, network not up; paced as counted, next run clean. No trip, no keyring warning. Fewer sends than the cadence × duration estimate because the closed laptop slept most of both days — which is what produced the evidence. `runs/2026-08-25-r8b/` (journal, daemon log, run log, `soak-check.txt`, `pmset` sleep/wake log) |

| 2026-08-30 | 11 | `227fca80` | **pass** for H1, H2 (H0 missed, H3 not run) | A 2/1/2, B 1/1/2 = 9 | 0 | pids 43950 (A, account 1) and 44048 (B, account 2), both `ACQ_NO_KEYRING=1`, 03:03–03:07 UTC. **H1 confirmed**: B's HEAD `/character` at 03:05:12.8, **4.1 s after A's counted GET**, reported `0:10:0,0:300:0`, and B's GET was answered `1:10:0,1:300:0` — `Account` rules count per account, not per IP or client. **H2 confirmed**: both daemons' GETs went out **28 ms apart** (03:06:03.835 / .863), each answered `1:10:0,2:300:0`, `wait_ms` 0, no HEAD, neither saw the other's hit. H0 unsampled: the two code exchanges were 31.2 s apart, so A's token hit had just aged out (`1:30:0` on both); consistent with N33, proves nothing new. H3 not run: daemon A re-logged in as account 2 at 03:06:43 (`logged in as` the second account, no HEAD, no re-probe) but was stopped 13 s later before `acq characters` — the carry-over stays a mock-only observation. Zero non-2xx, no trip. `runs/2026-08-30-r11/` (journals + daemon logs) |

| 2026-08-30 | first contact: `profile` | `fdb2d20f`-dirty | **halted by tripwire at send 2** | 1/1/0 | 0 × 429; **1 × 403 on HEAD** | Fresh daemon, second account, `acq profile` 27 s after login: the first-use **HEAD `/profile` was answered 403**; the GET never went out. No headers or body recorded — a HEAD has no body, and the daemon did not yet log a probe's response headers; the journal was not written because its directory did not exist (both fixed the same day: header snapshot on every non-2xx, journal directory created on demand). Two candidate causes, undecided: `/profile` does not accept HEAD (the N24 allowance has only been seen on `/character` and `/stash`), or the `account:profile` scope was not granted. Not a Cloudflare burst (2 sends). `/profile` now skips the probe; the next sample is one GET with ceiling 1 (token still valid). Binary provenance rule was not met (`-dirty`); does not affect the finding. |

| 2026-08-30 | first contact: `characters` (machine check after the 403) | `ad349ed0` | **pass** for its question; GET refused by the ceiling (my arithmetic: a fresh daemon always spends a token POST, so 2 was one short) | 1/1/0 | 0 | pid 92791. Refresh POST 200 (`token-request-limit` Ip `1:30:0`); **HEAD `/character` 204**, `character-list-request-limit` `0:10:0,0:300:0`: the token, the account, HEAD-in-general and the machine are all fine — the 403 was specific to `/profile`. Ceiling halt, not a trip. |
| 2026-08-30 | first contact: `profile` (second attempt, no probe) | `ad349ed0` | **endpoint works; our side rejected the answer** | 1/0/1 | 0 | pid 93065. Refresh POST 200; **`GET /profile` 200 with no `X-Rate-Limit-*` headers at all** (journal `rate {}`), 136 ms. The limiter's strict observation classed it "rate-limit protocol failure: missing x-rate-limit-policy"; the job failed and the body (with the uuid) was discarded. Scope is fine. Together with the HEAD 403: `/profile` is served differently from the API-policy endpoints — no rate-limit headers, no HEAD. New ground-truth observation for master-side; design decision needed (below) before the next call. |

| 2026-08-30 | first contact: `leagues` | `7a61c554` | **halted by tripwire at send 2 — our route was wrong** | 1/1/0 | 0 × 429; **1 × 403 on HEAD** | pid 97497. Refresh POST 200; **HEAD `/league` 403** with `www-authenticate: Bearer realm="pathofexile:production", error="insufficient_scope"` (via Cloudflare, `cf-ray a33505ad2f663aa8-DFW`, JSON content-type) — the header snapshot added after the `/profile` 403 classified it in one send. `/league` is the public league list and needs `service:leagues`; the account's leagues are `GET /account/leagues[/{realm}]` under `account:leagues`. Route fixed the same day; the tripwire cause now names an auth error when `WWW-Authenticate` is present. Not a rate-limit finding. |

| 2026-08-30 | first contact: `leagues` (rerun, `/account/leagues`) | `5cee50a5` | **pass**, with a new fact | 1/1/1 | 0 | pid 98647, after `reset-tripwire` on the persisted mark. Refresh POST 200. **HEAD `/account/leagues` answered 200 (not 204) and was counted**: state `1:10:0,1:60:0` straight after it, then the GET `2:10:0,2:60:0` — N24's uncounted HEAD is per endpoint, not a property of the API. Policy **`league-request-limit`, `Account`, `5:10:60,10:60:300`** (the mock had guessed `2:10:60,5:300:300`; corrected). No pacing error — headers are post-increment and trusted — but the probe costs what the GET costs, so `league` is now a no-probe route. Ground truth for master: the policy shape, and "HEAD counts on `/account/leagues`". |

| 2026-08-30 | first contact: `character <name>` | `9de33eec` | **pass** | 1/1/1 | 0 | pid 4900. Refresh POST 200; **HEAD `/character/{name}` 204, uncounted** (`0:10:0,0:300:0` after it — the N24 pattern, unlike `/account/leagues`); GET 200 `1:10:0,1:300:0`. Policy **`character-request-limit`, `Account`, `5:10:60,30:300:300`** — matches the C++ capture in ground truth exactly; 180 ms end to end. The "real but unsampled" list is now empty. |

| 2026-08-30 | persistence check | `ee494e3249c3` | **pass** | L1 1/2/3 = 6, L2 1/1/3 = 5 | 0 | pids 44737/44828, driven by `tools/persist-check.sh` (account `GERWARIC#7694`). Ceiling halt at send 6 left 3 `stash` children waiting and the parent held, none failed; **kill -9 mid-halt**; the successor restored the queue and its probe answered **`0:10:0,2:300:0` before any GET** — lifetime 1's own two stash hits, read from GGG's counters by a daemon that never sent them (the restart-replay premise, live). Remaining children resumed under the fresh ceiling; parent finished **done across two daemon lifetimes**; result served after. One requested id was absent from the 322-tab list (a typo picking ids) and reported in `unknown_tab_ids` — 5 children by input, not a drop. Zero non-2xx. `runs/2026-08-30-persist/` |

| 2026-09-01 | tracer | `7cc77a252d38` | **pass** — loop closed in 2 cycles | L1 1/0/1 = 2, L2 1/2/6 = 9 | 0 | pids 37648 (login) / 37964 (cycle 1), 23:52–00:04 UTC, driven by `tools/tracer-rung.sh --account GERWARIC#7694 <5 ids>` (Dump Tab, Maps (Remove-only), Uniques 1 (Remove-only), Winter Orb (Remove-only), the folder child 3.12 Pathfinder — the persistence check's five). Login: code-exchange POST 200, `GET /profile` 200, uuid recorded, bound reached at 2. Policy revision 1; offline plan = stale listing + 5 fetches, 6 req / 6..18 wire, `no quote` with the socket dead. Cycle 1 (ceiling 9): the plan **quoted** with the daemon up (the `/profile` discriminator residual did not bite), envelope identical to the offline one; HEAD `/stash/Standard` 204 `0:15:0,0:60:0` and HEAD `/stash/Standard/{id}` 204 `0:10:0,0:300:0` (both probes 0 hits, standing rule met); listing GET `1:15:0,1:60:0`; 5 stash GETs `1..5:10:0` in 0.5 s; parent `done`, 6/6 children done; bound reached exactly at send 9, nothing refused. Cycle 2 offline: `nothing to do` (5 covered tabs fresh); no-op apply `requests: 0`, no daemon appeared. Readback: all 5 tabs `fetched 59s ago`; **64 substashes** under the map/unique tabs in the store and outside the id list (the predicted uncovered-discovery observation); **0 item events** — nothing in those tabs changed since 2026-08-30. `verify.sh` reproduces the verdict. Friction notes in the rung section. `runs/2026-09-01-tracer/` |

| 2026-09-02 | tracer (rerun after the handle ruling) | `3d685c6d6603` | **pass** — loop closed in 3 cycles | L1 1/1/64 = 66, L2 1/2/6 = 9 | 0 | pids 45022 / 45233, 00:54–01:07 UTC, the same five ids, policy revision 2, no login (uuid on record). **Cycle 1: the 64 substashes** (46 under Maps (Remove-only), 18 under Uniques 1) already on record from the first run, planned through their parents — the first live discovery sample under "a policy id covers its children"; quoted; probe `0:10:0,0:300:0`; 15 GETs per 10 s, then holds of **14.98 s, 343.9 s, 14.84 s, 343.9 s** (rungs 7b and 10 exactly), every first send after a hold answered `1:10:0,…`, windows peaked at `15:10:0,30:300:0` and never exceeded; 64/64 children done; bound reached at 66. **Cycle 2: a refetch cycle** — the listing and the five parents were 51 min old at the start and the 13-minute cycle carried them past the 3600 s window (4033 s at compile), so the plan re-listed and re-fetched the five (6 req, ceiling 9); **its stash probe read `0:10:0,4:300:0` — cycle 1's last four GETs, verified as ours by the per-window bound: the verifier's nonzero-hit branch seen live for the first time**; the listing probe 0 hits; all 2xx. Cycle 3 empty; no-op with no daemon. Readback: 64 children of the selected tabs on record, 0 never fetched; items 168 → 816, 648 item events (all `added`, the substash contents). Zero 429, no trip. `verify.sh` reproduces. Observation for the rung: the driver's window guard compares the cycle's duration to the window, not the covered facts' **age at start** plus the duration — that gap is what bought the refetch cycle; the loop still closed. No owner friction notes entered. `runs/2026-09-02-tracer/` |
| 2026-09-02 | characters sample (guardian shape) | `953be323af82` | **pass** | 1/2/2 = 5 | 0 | pid 51350, 02:00 UTC, account `GERWARIC#7694`, `acq characters` then `acq character TheAbsenceOfPatience` under `ACQ_TRIPWIRE=1 ACQ_MAX_SENDS=5`; ceiling reached exactly at send 5, no trip. Both probes **0 hits** (`character-list-request-limit` `2:10:60,5:300:300`, `character-request-limit` `5:10:60,30:300:300` — N40 and the rung-3 sample, unchanged); both GETs `1:10:0,1:300:0`. Not a first contact — the **shape evidence the characters design needed** (CONTEXT.md, "Characters in the refresh plan"): the list (59 entries, every one `realm: "pc"`) carries exactly `class current experience id league level name realm` per entry — `id` is the 64-hex form and equals the fetched body's; league names include ended leagues (Ancestors, Phrecia 2.0, an SSF gauntlet event) with **no `expired` flag** on any entry, no `deleted` anywhere, one `current`; the fetched body carries a **`guardian` array of 5 items**, all with ids, none `lockedTo*`, whose `inventoryId` values (`Helm`, `BodyArmour`, `Gloves`, `Boots`, `Weapon`, `x`/`y` 0) are the character's own equipment slot names — an item's json cannot say which array it came from; `_split` equipment 17 / inventory 9 / jewels 7, guardian left in the envelope (not yet lifted); all 65 lifted items carry `frameTypeId` beside the deprecated `frameType`, none carry `realm`. `runs/2026-09-02-characters/` |
| 2026-09-02 | characters rung, pc (row 1) | `13fead8d4933` | **pass** — loop closed in 2 cycles | L1 1/4/112 = 117 | 0 | pid 1136, 15:08–15:22 UTC, `tools/tracer-rung.sh --account GERWARIC#7694 --characters all <the tracer's 5 ids>`, policy v3 revision 3. Offline plan: both listings 13 h stale → 1 stash listing + 5 tabs + 64 substashes + 1 character listing (pc, realm-wide) + **41 Standard characters** (of 59 listed; the other 18 sit in six other leagues, outside a Standard policy — not planned and not "skipped"), 112 req, 112..336 wire; quoted with the daemon up, envelope identical to the offline one. Ceiling 117 = POST + 4 probes + 112. **All four probes 0 hits** (`stash-list` `0:15:0,0:60:0`, `stash` `0:10:0,0:300:0`, `character-list` `0:10:0,0:300:0`, `character` `0:10:0,0:300:0`; every policy unchanged from the sample), 112 × 200, bound reached exactly at send 117, nothing refused; apply parent `success`, 112/112 children done. **The two facets paced independently**: stash 69 GETs with ~15 s holds after 15 and ~343 s after 30 and 60, done 15:22; characters 41 GETs with ~15 s holds after every 5 (its short window is 5 per 10 s) and a **280 s** hold after 30 (its 300 s window had opened before the 30th send, so the hold ended at the window's expiry, not 343 s), done 15:16 — the cycle lasted the longer facet, ~13 min. Cycle 2 offline: `nothing to do` (69 tabs and 41 characters fresh), no-op apply `requests: 0`, no daemon appeared. Readback: `acq store characters --realm pc` 59 rows, all 41 Standard fetched, none unfetched; **10 Standard characters (L86–L100) fetched with 0 items** — genuine, not a lifting gap: their bodies carry empty `equipment`/`inventory`/`jewels` (`_split` 0/0/0), stripped characters; all 41 bodies `metadata.version` `3.29.3`; 1081 `added` events, every one at `character/pc/<id>`, 0 stash events (nothing in the five tabs changed); guardian lifted for 4 characters (18 items, container `guardian`). One note: the first probe (HEAD `/stash`) waited 4.5 s behind the token POST, the other three 130–180 ms. `verify.sh` reproduces the verdict; the owner entered no friction notes. `runs/2026-09-02-tracer-150553/` |
| 2026-09-02 | first contact: `/character/poe2` (characters rung row 2, cycle 1) | `6caa07bfd1dc` | **pass** | 1/1/1 = 3 | 0 | pid 4418, 15:44 UTC, `tools/tracer-rung.sh --account GERWARIC#7694 --realm poe2 --characters all none`; policy v3 revision 4 (poe2 only — `policy set` replaces the whole policy, so the pc row's is gone until set again). **HEAD `/character/poe2` 204, uncounted** (`0:10:0,0:300:0` after it — the free probe, as pc); GET 200 `1:10:0,1:300:0`. Policy **`character-list-request-limit-poe2`**, `Account`, `2:10:60,5:300:300` — pc's windows under a poe2-suffixed name, so the two realms' listings keep separate counters (N6: a different name is different state). The list: 6 entries, every one **`realm: "poe2"`** (the mock's hypothesis; the docs' `pc|xbox|sony` is wrong for PoE2), the same seven keys as pc, 5 in `Standard` and 1 in `Runes of Aldur`, one `current`, no `deleted`/`expired`; none of the six ids or names is in the pc list (pc rows 59 before and after, 6 poe2 rows, nothing retired — a realm-R listing retires only realm-R rows, as designed). `runs/2026-09-02-tracer-154145/` |
| 2026-09-02 | first contact: `/character/poe2/{name}` (row 2, cycle 2) | `6caa07bfd1dc` | **pass on the wire; 4 of 5 bodies refused by the store** | 1/1/5 = 7 | 0 | pid 4923, 15:48 UTC, ceiling 7 exact, nothing refused by the rails. **HEAD `/character/poe2/pilatesinstructor` 204, uncounted**; 5 × GET 200, state `1..5:10:0`. Policy **`character-request-limit-poe2`**, `Account`, `5:10:60,30:300:300` — again pc's windows under the poe2 name. Then the finding the rung section predicted: the store refused four bodies (pilatesinstructor L33, flamdomrando L16, boomsplam L15 — Witches — and gerwarsmash L4 Warrior) with `missing an id on an item`; those children failed, the apply parent failed, the driver aborted (daemon stopped by hand with every job done or failed; journal and log slices copied into the run directory). The fifth, gerwarshot L5 Ranger, landed: `equipment` 8, `skills` 4 (+3 socketed supports), no `inventory` key, `metadata.version` `4.5.4f`, **every item with an id, the default `Bow Shot` attack included** — so `skills` items *can* carry ids. Which array carried the id-less ones is unknown: a refused body was dropped whole, and the job result keeps only the error string. Fixed the same day (facts v7: `refused` keeps the body verbatim, the error names `array[index]`, `acq store refused`) and rerun on the next row. |
| 2026-09-02 | `/character/poe2/{name}` rerun after facts v7 (row 2, cycle 2 again) | `b9e236eb298e` | **pass on the wire; 4 of 4 bodies refused — and kept** | 1/1/4 = 6 | 0 | pid 9749, 16:02 UTC, ceiling 6 exact; HEAD 204 uncounted, 0 hits; 4 × GET 200. The four refused bodies are in `refused` 1–4 and read: **every id-less entry is an item-granted skill.** A PoE2 weapon or shield that grants a skill (Rattling Sceptre → Skeletal Warrior, Attuned Wand → Mana Drain, Withered Wand → Chaos Bolt, Splintered Tower Shield → Raise Shield) carries that skill as a gem-shaped entry (`frameTypeId: "Gem"`, `support: false`) in its `socketedItems` with **no `id`**, while the host's own `sockets` is `[]`; the identical object appears again as `skills[0]` (deep-equal, no `inventoryId`); and a real support the player socketed into the granted skill (Meat Shield I, itself with `sockets`) is id-less too — the whole subtree under a granted skill has no ids. Nothing else lacks one: a rune socketed in a Focus (Desert Rune, `frameTypeId` `Currency`, `sockets[].type` `rune`) has an id, and gerwarshot's Crude Bow grants nothing, which is why it landed. Also read: **every PoE2 item carries `realm: "poe2"`** (pc bodies never carry the field, 0 of 1962 rows); `skills` entries carry `inventoryId` `DefaultAttackSkills` / `SkillSlots` (supports none); bodies `metadata.version` `4.5.4f`. Daemon stopped by hand, all jobs done/failed; `runs/2026-09-02-tracer-160139/` (journal, log, `refused-1..4.json`). What the store should do with a granted skill is an owner ruling (CONTEXT.md, "Characters in the refresh plan", open). |
| 2026-09-02 | characters rung, poe2 (row 2, closing run after the granted-skill ruling) | `21f1c515b39b` | **pass** — loop closed in 2 cycles | L1 1/1/4 = 6 | 0 | 16:14 UTC, the same invocation; listing fresh (30 min), gerwarshot fresh, plan = the four refused characters, ceiling 6 exact; HEAD 204 uncounted 0 hits, 4 × GET 200; apply `success` 4/4. All four landed: pilatesinstructor 30 items, flamdomrando 22, boomsplam 20, gerwarsmash 11 (83 `added` events at `character/poe2/<id>`); **8 granted skills left in place** (`_granted` `{equipment: 1, skills: 1}` per character; `store status` `granted: 8`), no refusal, no drift. Cycle 2 offline `nothing to do` (5 covered characters fresh; Runes of Aldur outside the policy), no-op apply, no daemon. Readback `acq store characters --realm poe2`: 6 rows, 98 items, every Standard character fetched. `verify.sh` reproduces the verdict. Order-of-work (5) is complete: both poe2 routes sampled and recorded, the shape that blocked ingest ruled and built the same day. `runs/2026-09-02-tracer-161419/` |
| 2026-09-02 | legibility run — the refresh slice read from the terminal (pc, five tabs + all characters; the characters-row shape rerun on the legible-output build) | `695c1ec131f9` | **pass** — loop closed in 2 cycles; owner verdict in the section below | L1 1/4/112 = 117 | 0 | pid 26435, 19:04–19:16 UTC (cycle 1, 12 min); policy revision 7 (`5ba2e1880a,421496994e,0b4c8308d2,ad19966e17,3351947d46` + `characters: all`, 3600 s). Every fact 3 h old, so one plan: 1 stash listing, 5 tabs, 64 substashes (46 + 18), 1 character listing, 41 characters — the first live plan rendered **grouped** (`fetch 64 substashes under 2 of those tabs`) rather than as 112 lines. Ceiling 117 exact; four probes, each 204 with 0 hits; 112 × GET 200; quoted (the fresh-daemon `no ETA until its policy is learned` shape). Holds: character 8 (7 × ~15.5 s, one **280 s** after the 30th), stash 4 (2 × ~14.7 s, 2 × ~343.7 s) — two policies pacing independently, as row 1 saw. Apply `success` 112/112; `store_changes` all zero over 112 responses (nothing moved on the account). Cycle 2 offline `nothing to do` (69 tabs and 41 characters fresh), no-op apply with no daemon. Readback footers: `402 tabs: 69 fetched, 317 never fetched, 16 folders (never fetched); 816 items`; `59 characters: 42 fetched (10 with empty bodies), 17 never fetched; 1146 items`; `0 events in the last 1.4 h`. No friction notes typed at the prompts; the owner's verdict was given to the agent and recorded later the same day. `runs/2026-09-02-tracer-190328/` |
| 2026-09-03 | density validation (attempt 1) | `3e6757dd` | **aborted before apply; no sends** | 0/0/0 | 0 | The compact renderer's one-line `quote:` was present in the envelope but the driver still extracted only the old `quote (` form, so the owner stopped at approval and the daemon was stopped; fixed by `efc70288`. Partial evidence: `runs/2026-09-03-tracer/`. |
| 2026-09-03 | density validation — pc, five tabs + all characters | `efc70288bbe8` | **pass** — loop closed in 3 cycles; owner verdict below | L1 1/4/112 = 117, L2 1/4/112 = 117 | 0 | pid 67677 / 75012, policy revision 9. The 112-request decision view named the five parents, counted 64 substashes and 41 characters, stated one 7 h stale reason, and showed the compact quote before approval. Both cycles: four probes at 0 hits, 112 GET 200, exact ceiling, apply `success` 112/112, no changes. An ~8 h pause after cycle 1 aged its facts past the 1 h window, so cycle 2 correctly repeated the plan; cycle 3 was a no-op with no daemon. The verifier reproduced every send and hold; no friction notes were typed. `runs/2026-09-03-tracer-024735/`. |
| 2026-09-04 | price-notes run — the owner's `ACQUISITION-PRICE-TEST` tab and the forum-listed character I_EXIST; three ad-hoc jobs on one daemon (pricing slice, `brainstorming-notes/12`) | `2e347e16c0fc` | **pass** — both observations landed | 1/3/3 = 7 | 0 | pid 31313, 00:51–00:52 UTC, ceiling 7 exact (the halt is logged as the seventh send goes out; nothing refused). Three probes, each 204 with 0 hits: `stash-list-request-limit` `10:15:60,30:60:300`, `stash-request-limit` `15:10:60,30:300:300`, `character-request-limit` `5:10:60,30:300:300`, all as recorded. GET `/stash/Standard` (response 541; the listing now names the test tab `03cb479c65`, idx 56, `public: true`), GET `/stash/Standard/03cb479c65` (response 542, 80 items, 50 with notes), GET `/character/I_EXIST` (response 543, 53 items). Read: **`/character` carries no `forum_note`** on the body armour the owner linked and priced in a forum post that the trade site lists — a forum listing is not observable through the character endpoint; and the in-game price dialog's vocabulary, **39 distinct currency words**, verbatim in the run's `notes-check.txt` and `brainstorming-notes/12` E21 (`chrome`, `jewellers`, `fusing`, `exalted` where the C++ table had `chrom`, `jew`, `fuse`, `exa`; no `chisel`, `coin` or `silver`; 23 words the table lacks). Traps: the daemon's exit left 1.1 MB in the facts file's WAL with no final checkpoint, so `census.py`'s guard refuses the file until something checkpoints it (`notes-check.py` reads through it); the owner copied the daemon log instead of the journal — the journal is `<socket dir>/acquisition-playground.ggg.sends.jsonl`, not `<socket>.ggg…`; the run directory now holds the journal slice, the pid's log slice and the tool's output. `runs/2026-09-04-price-notes/` |

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

## Rung 11 — two accounts, one machine (run 2026-08-30: H1, H2 confirmed)

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

Run 2026-08-30 (ledger row above): **H1 and H2 hold** — per-account
counters, no cross-account interference between two daemons on one IP.
H3 (the account-switch carry-over on one daemon) was not sampled; it is
predicted from the code and seen on the mock, is conservative
(over-waiting only), and is not worth a further live run on its own.
H0 was missed by one second and is already covered by N33.

## Persistence check — halt, crash, resume (run 2026-08-30: pass)

The persisted queue (`daemon.db`, CONTEXT.md decision 2026-08-30) is
proven offline and against the mock; two of its behaviors rest on a wire
premise and got one rung-style check each, in a single run of ~13 sends:
a halted queue resuming under a successor daemon (rail 5's ceremony
rehearsed for real), and a restore that sends nothing before its probe
has read the account's real counters. `tools/persist-check.sh
[--account NAME] <6 tab ids>` drives it from one terminal and `--mock`
rehearses the identical flow.

Run 2026-08-30 (ledger row): **pass**, every check green. The one
surprise was input, not behavior: a requested tab id was absent from the
322-tab list (mistyped when picking ids); the parent refreshed the other
5 and named the stranger in its payload's `unknown_tab_ids` — the
honest-success shape doing its job. The evidence line: lifetime 2's
probe answered `0:10:0,2:300:0` before any GET — lifetime 1's two stash
hits, read back from GGG's counters by a daemon that never sent them.
The rail-5 ceremony was exercised as written and nothing in the run
argued for more.

## Tracer rung — policy → plan → apply → replan (run 2026-09-01 and 2026-09-02: pass)

The refresh tracer's step 9: the owner's first real use of the slice on
the real account. It asks nothing new of GGG — every route it touches is
well-trodden — so no hypothesis document: `tools/tracer-rung.sh
[--account SEL] [--league L] [--max-age S] [--cycles K] [--characters
all|ids] <tab1,...|all>` drives it (`tools/tracer-verify.py` verifies the
journal; `--mock` rehearses the identical flow, exact ceilings included),
and the ledger row is the record. Each cycle runs on a fresh daemon whose
ceiling is derived from the plan exactly, so the ceiling halt right
after the last planned send is a cycle's expected end; the driver
refuses a stale binary, working-tree changes to the rung's own files,
leftover isolation env, and a running daemon. Holds on
`stash-request-limit` (`15:10:60, 30:300:300`): none while `n ≤ 15`, one
~15 s hold before the 16th, ~343 s holds above 30 (a hold ends at the
window's expiry, so 343 s is the worst case, not a constant — N45).

What the two runs taught became rulings in `CONTEXT.md` (binding
confirmed; a policy id covers the tab and its children; the method-test
verdict) and observations in `REFRESH-SLICE.md`.

### Friction notes (owner)

Data the way the send journal is data. The driver prompts after every
phase and writes `friction.md` in the run directory; across the four
runs so far the prompts collected one note, and every verdict arrived
in conversation with the agent — so the owner's words are recorded
here verbatim from wherever they were said.

Notes (owner, 2026-09-01):

- **logging in**: I had to enter my keyring password twice.

The owner's only other remark, made to the agent rather than into the
prompt: the driver's output is dense and confusing to a human reader,
and that is accepted for now — the agent reads it, and wordsmithing the
output before the tracer has proven its worth would be wasted effort.

## Characters rung — the refresh plan with characters (run 2026-09-02: pass, both rows)

Order-of-work steps (4) and (5) of the characters ruling (`CONTEXT.md`):
the same driver, rails and verifier, with the character facet in the
policy. Two invocations, both the owner's, from a terminal, under the
standing rule:

```sh
tools/tracer-rung.sh --account GERWARIC#7694 --characters all <tab1,...>       # row 1, pc
tools/tracer-rung.sh --account GERWARIC#7694 --realm poe2 --characters all none  # row 2, PoE2 first contact
```

Row 1 closed its loop in one 112-request cycle (both facets paced
independently; ten stripped characters were a real shape). Row 2 was
the standing rule's first-contact shape on both poe2 routes, produced
the refused-body finding (four of five bodies, kept since facts v7) and
the granted-skill ruling, and closed after the rerun. Ledger rows
`characters rung, pc`, the three `/character/poe2` rows; claims
N41–N45; the findings in `REFRESH-SLICE.md`.

## Legibility run — the refresh slice read from the terminal (run 2026-09-02: pass; approved — density closed by the run below)

The characters-row shape rerun on the legible-output build (`695c1ec1`),
the owner at the terminal. Not a first contact; the ledger row and this
verdict are the record.

**Owner verdict (2026-09-02, given to the agent rather than typed at
the prompts; no `friction.md` exists for this run): the output is
approved, and it is "still dense, verbose output".** Density stays the
open item: cut words before adding structure, judged against the next
run a person reads live, not pre-fixed. `CONTEXT.md` carries the same
verdict in the ruling's closing paragraph.

## Density validation — the decision view read live (run 2026-09-03: pass; approved)

The owner read the compact 112-request plan before either live cycle;
the second cycle followed an eight-hour pause, not a planner failure.
The ledger row and the saved bundle are the wire record. The mock and
first live attempt exposed three text/driver integration defects, all
fixed before the passing run.

**Owner verdict (2026-09-03, verbatim, given to the agent rather than
typed at the prompts): "The only thing I want to change is printing
stash tab id's in the final column on the right similar to characters.
Everything else looks good."** Commit `28db97c6` makes that sole change;
C53's density validation is closed.

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

First contact under the standing rule, 2026-08-30 (run ledger): `/profile`
answers 200 with no rate-limit headers and 403 to HEAD; `/account/leagues`
is `league-request-limit 5:10:60,10:60:300` and counts its HEAD; `/league`
needs `service:leagues` (our route was wrong, fixed). The daemon carries
both endpoint facts as declared route knowledge. GGG answered the same
day (Q12): `/profile` is **not rate limited at present** — the
policyless declaration is confirmed and stays until headers ever
appear — and the counted HEAD on `/account/leagues` is a defect **GGG
will correct in a future release**; until the free HEAD is observed
live, it is treated as counted, and the observation that shows the fix
is what deletes the no-probe declaration and restores the probe.
`/character/{name}` (same day) is the ordinary pattern: free HEAD, full
policy, the C++ capture's shape.

The multi-account build and the characters rungs (pc and PoE2) are
done; the frontier is pricing (`decisions/pricing.md`).
