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

Every job kind has had first contact as of 2026-08-30 (run ledger:
`profile`, `leagues`, `character`; `leagues` was routed to `/league` until
that day). A new kind gets the same treatment under this rule. What the
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

| 2026-08-25 | 10 (second run) | `5d792d0a` | **pass** (wire); diff false positive fixed client-side | 1/2/323 = 326 | 0 | pid 6111, 11:19–12:20 UTC, 61 min; fresh daemon, probes 0 hits; identical to the rerun on the wire: ten ~343 s holds, `1:10:0,1:300:0` after each, max 15/30, zero non-2xx, no keyring warning. Snapshot 322 tabs / 18 072 items / 0 errors; no tab or item added, removed, or moved. Reported **10 items changed** — all `veiledMods`, whose placeholder ids GGG re-randomizes per fetch (`Prefix06` → `Prefix01`); not stash changes. The diff now ignores that field (`pull.rs`, `VOLATILE_ITEM_FIELDS`); new ground-truth observation for master-side. `runs/2026-08-25-r10c/` |

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

## Rung 11 — two accounts, one machine (hypothesis 2026-08-29, run 2026-08-30: H1, H2 confirmed)

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

## Persistence check — halt, crash, resume (run 2026-08-30: pass)

The persisted queue (`daemon.db`, CONTEXT.md decision 2026-08-30) is
proven offline and against the mock; two of its behaviors rest on a wire
premise and get one rung-style check each, in a single run of ~13 sends:
a halted queue resuming under a successor daemon (rail 5's ceremony
rehearsed for real), and a restore that sends nothing before its probe
has read the account's real counters. This asks nothing new of GGG — the
question is about our restart behavior — so no hypothesis document; this
section plus a ledger row is the record. The halt trigger is the
**ceiling**, not the tripwire: tripping the tripwire live requires a real
violation, and a ceiling halt exercises the same halt-leaves-jobs-waiting
machinery with zero violations (`reset-tripwire` cannot release a ceiling
halt mid-lifetime anyway — the send count survives it — so restart is the
release, which is the thing under test).

Rails on throughout; binary provenance rule; journal at its default path,
copied to `runs/2026-08-30-persist/` after. Pick 6 small tabs from
`acq tabs` (a store read; no daemon). `ACQ_IDLE_SHUTDOWN=600` on
lifetime 1 so the halted daemon is still there to kill; if it idles out
before step 3, note it in the ledger and continue — the idle-out variant
is equally valid (the queue is on disk either way).

`tools/persist-check.sh [--account NAME] <6 tab ids>` drives the whole
table from one terminal (the selector is required with more than one
persisted account — every job carries an account): it refuses stale
binaries and leftover env, spawns each daemon
with the step's rails, gates every wire phase on an explicit enter,
aborts loudly on a tripwire trip or a failed job, verifies the
expectations from the journal (probe before first send per route, no
non-2xx, two lifetimes), collects the evidence into the run directory,
and drafts the ledger row. `--mock` rehearses the identical flow against
the mock — run green 2026-08-30 (L1 `1/2/3`, halt at 6, 4 children
waiting, parent done under the successor) — with one caveat: the mock
provider dies with the daemon, so only the live run can show the probe
reading counters that survived the kill. The table stays the
specification; the script implements it.

| Step | Command | Expect | Stop if |
| --- | --- | --- | --- |
| 1 | `ACQ_GGG=1 ACQ_TRIPWIRE=1 ACQ_MAX_SENDS=6 ACQ_IDLE_SHUTDOWN=600 acq refresh --tabs <6 small tabs>` (blocking; note the parent id it prints, Ctrl-C once the halt lands — a disappearing client cancels nothing, by decision) | POST, HEAD+GET list, HEAD stash (probes report 0 hits), then child GETs until the ceiling halts at send 6 — the 2-wide gate may let one extra land; 3–4 children never sent | probe hits > 0 (standing rule: something else is on this account); any non-2xx |
| 2 | `acq jobs` (after ~10 s) | remaining children `waiting`, parent held, nothing running; `daemon status` names the ceiling halt | any child `failed` for lack of a send (the pre-persistence behavior) |
| 3 | `kill -9 <daemon pid>` (pid from `acq daemon status`) | daemon gone mid-halt, no shutdown path run; queue on disk | |
| 4 | `ACQ_GGG=1 ACQ_TRIPWIRE=1 ACQ_MAX_SENDS=10 acq dash` (any spawning command works) | successor restores the queue and journals: open line, POST, **HEAD stash probe before any GET**, the probe reporting step 1's hits still inside the 300 s window — **hits > 0 is expected here and only here**; they are ours and this run knows it, so the standing rule's "stop and find it" does not apply — then the remaining child GETs; parent finishes **done** across two daemon lifetimes, not interrupted (its held result predates the kill) | any GET before its route's probe; parent finishes interrupted (held-result ordering differs live from the mock); any 429 or non-2xx |
| 5 | `acq result <parent-id>`, then `acq daemon stop`; copy the journal; ledger row | result served by a daemon that never ran the job | |

Expected totals: lifetime 1 `1/2/3` = 6 (+1 slip), lifetime 2 `1/1/3–4`
≤ 6; ceiling 6 then 10; every send on well-trodden routes
(`stash-list-request-limit`, `stash-request-limit`), max ~7 stash GETs in
300 s against a 30-per-300 s window. Residual stated plainly: at kill
time nothing is mid-send (the halt guarantees it), so the
running-job-re-queued-as-duplicate path stays mock-proven; what this run
checks live is the premise that path rests on — a restore's first send on
a route happens after its probe has read GGG's real counters. Step 3→4 is
also rail 5's ceremony (`acq jobs`, cancel what should not resume, then
respawn) run for real; if it feels sufficient in practice, rail 5 stands
as written and the ceiling-doesn't-persist caveat needs no further rule.

Run 2026-08-30 (ledger row): **pass**, every check green. The one
surprise was input, not behavior: a requested tab id was absent from the
322-tab list (mistyped when picking ids); the parent refreshed the other
5 and named the stranger in its payload's `unknown_tab_ids` — the
honest-success shape doing its job. The evidence line: lifetime 2's
probe answered `0:10:0,2:300:0` before any GET — lifetime 1's two stash
hits, read back from GGG's counters by a daemon that never sent them.
The rail-5 ceremony was exercised as written and nothing in the run
argued for more.

## Tracer rung — policy → plan → apply → replan (prepared 2026-09-01; run 2026-09-01: pass)

The refresh tracer's step 9 (`CONTEXT.md`, "Annotations & plans"): the
owner's first real use of the slice on the real account. It asks nothing
new of GGG — every route it touches is well-trodden
(`stash-list-request-limit`, `stash-request-limit`, the token endpoint),
and the `apply` kind is a fan-out parent of kinds that have had first
contact — so no hypothesis document: this section, a ledger row, and the
friction notes below are the record. What it collects, on purpose:

1. **The plan's projection against the wire.** A plan states its own
   wire estimate (`min..max` sends plus named prerequisites: one probe
   per route this lifetime, a token refresh). Each cycle runs on a fresh
   daemon whose ceiling is *derived from the plan, exactly*: one token
   POST, one HEAD per route the plan touches, one GET per action. The
   rails trip the moment the count reaches the ceiling (`rails.rs`,
   `sends >= max`), so the daemon halts on the bound right after the
   last planned send — that halt is the expected end of a cycle, and
   it is checked as such after every wire phase: the daemon must report
   the tripwire armed, a ceiling equal to the plan's, exactly that many
   sends counted, and a ceiling halt in force, with the journal
   agreeing — a journal count that merely matches is not evidence the
   rail was there. The bound is
   counted as responses land while the gate admits two sends in flight
   (rail 1's caveat), so it can be overshot by one already-dispatched
   send at most; either way a send the plan did not project fails the
   cycle — it consumes the bound and shows as a planned child refused,
   or it lands as a `ceiling + 1` journal the verifier rejects.
2. **The quote on the wire.** With a real daemon up, `refresh --plan`
   either carries its quote (headroom, queue, ETA per scope) or prints
   why not. The accepted residual is collected here: real GGG's
   `/profile` `name` may lack the `#discriminator` the session username
   carries, in which case the note reads "daemon quote rejected" — an
   observation to record, not a fault; the plan applies unquoted.
3. **What is applied is what was reviewed.** The envelope applied is
   the quoted file itself, checked to be the offline envelope (the
   ceiling's source) plus the quote — actions in order, listing basis,
   identity, policy revision, counts; the derived `age_seconds` inside
   reasons is ignored (it advances with the clock between compiles)
   while the reason kinds are compared — and its action list is
   rendered from that file and confirmed before the apply. Facts moving
   between the two compiles is a stop.
4. **Friction notes** — the product-side data (`CONTEXT.md`, "Binding-plan
   friction"; the method test). Taken at the moment, not recalled: the
   driver prompts after every phase and writes them to
   `runs/<date>-tracer/friction.md`; they are pasted into the
   subsection below.

Preconditions: the standing rule's rails and binary provenance; from a
terminal (keychain, browser). Two accounts are persisted, so every
command carries `--account GERWARIC#7694` (the selector resolves as
`acq` does: username or username without `#discriminator`, both
case-insensitive with Unicode lowercasing, or the exact uuid). That account's index entry predates uuid-at-login and has no
uuid; intent binds to the uuid, so the run opens with a re-login as the
same account (a code-exchange POST and the login's own `GET /profile`,
ceiling 2, exactly). One fresh daemon per wire phase, stopped when the
phase is over, `ACQ_NO_SPAWN=1` throughout: the offline claims (plan
compiled with no daemon; an empty plan's apply contacts nothing) are
checked with the socket dead. The league's listing on record is from
the persistence check (2026-08-30), so the first plan is the stale
listing **plus** the selected tabs' fetches in one plan — the fetches on
the old listing's membership (D5a: a plan never expands itself; the new
listing's facts land for the next plan).

**Selection decides the shape of the run.** The planner matches policy
ids exactly and a substash's id is its own (`TabSelection::selects`),
so an id list never covers the substashes a map/unique fetch discovers:
they land in the store, uncovered, and the loop closes after one
working cycle — the driver reports them as an observation ("n
substashes discovered under selected tabs are not covered by the id
list"). `all` covers them and runs the discovery cycle one plan later.
Whether "a parent covers its substashes" belongs in the policy shape is
a friction question for the owner, not something the rung pre-decides.
Pick the tabs from `acq --account GERWARIC#7694 tabs --league Standard`
(a store read; no daemon): a handful, including one map or unique tab
so the uncovered-discovery observation is real. `all` is allowed and is
the owner's call — 323 requests with ~343 s holds per 30, then every
substash a second cycle (a map tab may expose hundreds). Its freshness
window must outlive the cycle: at the default 3600 s the hour-long
cycle 1 (rung 10: 61 min) would leave its own listing and early fetches
stale for cycle 2, which would re-list and re-fetch instead of planning
only substashes, and the loop could never close. `all` therefore
defaults `max_age_seconds` to 86400, and the driver refuses any cycle
whose (deliberately over-estimated) wire duration is not at most half
the window.

`tools/tracer-rung.sh [--account SEL] [--league L] [--max-age S]
[--cycles K] <tab1,...|all>` drives the table from one terminal
(`tools/tracer-verify.py` is its journal verifier; `--self-test` runs
the synthetic journals that pin the nonzero-hit branches the mock
cannot reach). It
refuses a stale binary, working-tree changes to the rung's own files
(driver, verifier, control documents, crates — the ledger's tip must
name what ran), leftover isolation env, and a running daemon;
writes the policy from the selection; gates every wire phase on an
explicit enter; derives each cycle's ceiling from the offline plan;
checks and shows the envelope it applies; treats the bound reached
exactly after the planned sends as the cycle's end and any other halt
(a tripwire trip, a ceiling with sends missing, a child not done) as a
stop; reads the facts back and fails on a selected tab missing from the
store or a read that errors; verifies the journal; drafts the ledger
row; and prints the friction notes. `--mock` rehearses the identical
flow, exact ceilings included, against the mock — run green 2026-09-01
in both shapes: `all` = login `1/0/1`, cycles `1/1/1` (bootstrap
listing), `1/1/7` (seven tab fetches), `1/1/7` (seven substashes
discovered one plan later), then an empty plan applied as a no-op with
no daemon; ids `cur1,dump,maps` = login, `1/1/1`, `1/1/3`, then empty,
with four substashes under `maps` reported as uncovered. Every cycle
quoted; every cycle's sends equal to its plan plus probes plus the
POST. The table stays the specification; the script implements it.

| Step | Command | Expect | Stop if |
| --- | --- | --- | --- |
| 0 | preflight: `acq --version` = HEAD, the rung's files clean at HEAD, no `ACQ_*` isolation leftovers, no daemon; `acq accounts` | `GERWARIC#7694` persisted, no uuid | a daemon is up; the binary is dirty or stale; uncommitted changes under `tools/`, the control documents, or `crates/` |
| 1 | fresh daemon (ceiling 2), `ACQ_GGG=1 acq auth` in the browser as **the same account** | POST 200 (`token-request-limit` Ip `1:30:0`), `GET /profile` 200 with no rate headers (N38), `logged in as GERWARIC#7694`, the index now carries the uuid; bound reached at send 2; daemon stopped | the login lands as another account (intent would bind to the wrong identity); any non-2xx |
| 2 | `acq policy set '{"version":1,"leagues":{"Standard":{"tabs":[…],"max_age_seconds":3600}}}'` (the driver writes 3600 for an id list, 86400 for `all`, or `--max-age`), then `acq refresh --plan` with **no daemon** | revision 1; the plan is the stale listing (≈2 d) plus one fetch per selected tab, `n+1` requests, `n+1..3(n+1)` wire sends, the two prerequisites named; stderr `no quote: … plan compiled offline`; no daemon appeared | a daemon appeared; the note is anything but "no quote" |
| 3 (cycle 1) | fresh daemon, ceiling `1 + 2 + (n+1)`; `acq refresh --plan --json` again (quoted) — checked equal to the offline envelope plus the quote, actions shown; `acq refresh --apply=<that file> --max-requests n+1` | the quote, or the discriminator note (record which); journal: POST, `HEAD /stash/Standard` 204 reporting **0 hits**, `GET` list 200, `HEAD /stash/Standard/{id}` 204 reporting **0 hits**, `n` tab GETs 200; parent `done`, children `n+1` done, 0 failed; the bound reached exactly at the last send; daemon stopped | the run's **first** probe on a route reports hits > 0 (standing rule: something else is on this account); any non-2xx; a child failed (a tab gone since 2026-08-30 is data — record it, then stop); the ceiling halts with sends missing, or the journal shows `ceiling + 1` (a send the plan did not project, either way) |
| 4 (cycle 2) | plan offline again | **id list**: `nothing to do` — the substashes discovered under the map/unique tab are in `acq tabs` but outside the policy; `acq refresh --apply` with no daemon prints `requests: 0` and nothing appears on the socket. **`all`**: `fetch substash` lines, `m` requests, ceiling `1 + 1 + m`; the stash probe reports cycle 1's hits still inside each window — **ours, expected; the verifier bounds every reported window's hits by this run's own sends inside that window plus that window's timing bucket (N11/N12: 5 s on a rule's first window, 60 s on the later ones, as `bucket_for` in `ratelimit.rs`) at the probe's time, never by a cumulative total**; journal `1/1/m`; children `m` done | a daemon appeared on the no-op; hits beyond ours in any window; otherwise as step 3 |
| 5 (cycle 3, `all` only) | plan offline again | empty; no-op apply with no daemon | a daemon appeared; the plan still has work (record what and why; not a failure) |
| 6 | `acq tabs --league Standard`, `acq store status`, `acq store events --hours <the run's span + 1> --limit 1000000` (store reads; hitting the limit fails the run) | every selected tab `fetched` moments ago (an id the first plan reported as unknown is the one exception); item events from this run; the final `refresh --plan` empty when the loop was recorded closed | a selected tab is missing from the store; a read errors; a closed loop whose final plan is not empty |
| 7 | evidence: this run's slices of the journal and the daemon log, the plans, apply results, store reads, a copy of the verifier with checksums, and a `verify.sh` that re-runs the verification through that copy, in `runs/<date>-tracer/` (a repeat the same day gets a time suffix, never overwrites); ledger row; friction notes pasted below | the bundle's `verify.sh` does not reproduce the summary's verdict |

Expected totals for `n` selected tabs: login `1/0/1`, cycle 1
`1/2/(n+1)`, then nothing. `stash-request-limit` is `15:10:60,
30:300:300`, so with a zero-hit probe there is no hold while `n ≤ 15`;
`16 ≤ n ≤ 30` costs one ~15 s hold before the 16th (rung 7b); above 30
the ~343 s holds begin (rung 10). The listing GET is on its own policy
and never holds here. For `all`: cycle 1 `1/2/323` with ten ~343 s
holds, cycle 2 `1/1/m` for `m` substashes with its own holds (under
the day-long window; cycle 1's listing and fetches stay fresh), and
cycle 2's probe carries cycle 1's hits still inside the windows, which
the fresh daemon reads and paces on (it over-waits, never floods — the
seeding the quote also uses). Residuals stated plainly: the listing and the
fetches share one plan by design, so a tab renamed or removed since
2026-08-30 surfaces as a failed child or an `unknown_tabs` id — that is
D5a's eventual reconciliation being seen live, and a friction note, not
a rail failure; and the rung samples the slice, not the frontier — a
whole-league policy is a second run, if the notes ask for it.

### Friction notes (owner, filled during the run)

Data the way the send journal is data; each note is one line, taken when
it happened (`friction.md` in the run directory, pasted here). The
prompts are the questions the notes are for, not a form to complete:

- **Intent** — writing the policy by hand: were tab ids the right handle,
  or did you want names, types, "everything but", or "this tab and its
  substashes"? Did the uuid re-login and the `--account` selector cost
  anything?
- **Plan** — did the plan read as your intent? Was "listing + fetches in
  one plan" what you expected, or did the stale listing surprise you?
  The quote: useful, noise, or absent?
- **Apply** — the wait, the feedback during it, the `--max-requests`
  ceremony, the result payload: what did you want to see that you did
  not?
- **Replan / binding** — the discovered substashes an id list leaves
  uncovered, the two-cycle discovery under `all`, subset-only
  reconciliation, an empty plan as the closing signal: did any of it
  hurt? (D5a is revisable on exactly this.)
- **Facts** — reading the store back: did the tabs/events answer "what
  changed"?
- **Anything that made you reach for `acq refresh --tabs` instead.**

Notes (owner, 2026-09-01):

- **logging in**: I had to enter my keyring password twice.

The owner's only other remark, made to the agent rather than into the
prompt: the driver's output is dense and confusing to a human reader,
and that is accepted for now — the agent reads it, and wordsmithing the
output before the tracer has proven its worth would be wasted effort.

Agent observations from the same run (not owner friction; recorded so
the re-ruling has them):

- **Quote**: on a fresh daemon the quote can only say "no ETA until the
  policy is learned" for every scope, and every cycle here runs on a
  fresh daemon by design — so the quote was structurally uninformative
  in this rung. Whether it is useful in a long-lived daemon is untested.
- **Plan note**: the offline "no quote" line prints twice (once in the
  plan text, once as the driver's echo) and carries a raw
  `No such file or directory (os error 2)` where "no daemon running"
  would do.
- **Facts**: "what changed" was answered with 0 item events and five
  `fetched 59s ago` rows; the readback never says "5 tabs refetched,
  0 items changed" in one line. The folder child's row is indented and
  its name truncated, pushing the columns out of line.
- **Binding**: the 64 uncovered substashes were reported as predicted;
  the owner did not reach for `acq refresh --tabs` and wrote no note on
  whether "this tab and its substashes" is the handle wanted — that is
  the open question for the `CONTEXT.md` re-ruling, not answered by this
  run.
- **Keyring**: the debug binary is unsigned, so macOS Keychain treats
  every rebuild as a new program and prompts on first access; two
  prompts is two keychain operations (read the stored session, save the
  rotated token, or client and daemon each once). Not a stop; a
  rebuild will do it again.
- **Probe timing**: the two HEADs carry `wait_ms` 191 and 117 while
  every GET carries 0 — the probes queued behind the token POST that
  preceded them in the same lifetime. Consistent with the journal
  order; noted, not investigated.

Ruled on after the run, in `CONTEXT.md`: the "Binding-plan friction" open
topic (re-ruled or confirmed) and the method-test verdict that the pricing
session opens with.

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

The frontier is the frontend boundary and the multi-account build
(`CONTEXT.md`).
