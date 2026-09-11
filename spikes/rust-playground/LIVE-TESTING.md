# Live testing control

Control document for running the Rust daemon against the real GGG API:
the standing rule for first contact with anything new, the safety rails,
and the ladder's record (closed 2026-08-27). Every live run is one row in
`RUN-LEDGER.md`, its evidence under `runs/` (gitignored); the ledger and
the run sections were part of this file until 2026-09-10 (the file at
`4c9755c6`), so a pointer to "`LIVE-TESTING.md`'s run ledger" resolves
there. `CONTEXT.md` invariants apply. Ground-truth facts learned live go
to `docs/design/network-ground-truth.md` as numbered claims (authored
master-side, cherry-picked here). History lives in git: the blast-radius
review (risks R1–R8) and the L0 rails build with its review register
(L0-R1–R13) at `9fa99459`; the ladder's rung table, preconditions, soak
mechanics and postmortems at `944406ff`; the refresh rungs' preparation
tables and agent observations at `d660d1f5`, and what those runs taught
is `REFRESH-SLICE.md`. All of R1–R8 are resolved: R1 by the dead-grant
decision (C24), R2–R7 by the rails below, R8 fixed in `529bdd92` and seen
live once (rung 8).

The ladder's goal — confidence that the daemon **halts rather than
floods** — was met on 2026-08-27 (the record below). The ladder was the
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
  GET) as one row in `RUN-LEDGER.md` and as a ground-truth claim
  master-side. That row is what lets `ratelimit.rs`'s test table cite it.
  The row carries the evidence's location and where each lesson landed —
  a trap to the live-run skill, a fact to a claim, a verdict verbatim
  beside its ruling — never the lesson's text; that is what keeps the
  ledger a ledger.
- **Post-violation: wait 360 s, then `reset-tripwire`; never
  reset-and-retry.** After any tripwire trip write the cause in the
  ledger, wait at least 360 s (the longest observed policy window plus
  the 60 s bucket), and only then `acq daemon reset-tripwire`. A trip
  is evidence to read, not a retry prompt. This is a fact about GGG, not
  paperwork, and it outlives the ladder.
- **A failed probe leaves only headers.** A HEAD has no body, so a
  probe's non-2xx is classified from its response headers, which the
  journal (`headers`), the daemon log, and the trip cause all carry
  (since the `/profile` 403 of 2026-08-30). Routes known not to accept HEAD
  skip the probe (`route_probes` in `daemon.rs`) and are taught by
  their first GET instead.
- **Build before you run; the run record maps both binaries to HEAD.**
  Neither binary carries a commit. Two values say which code ran (C84):
  the shared-contract revision — a digest over the protocol and store
  crates (sources and manifests), the root manifest and the lock, which
  `acq version --json` prints as `contract` — and the
  daemon artifact, the SHA-256 of the `acqd` file, which the daemon
  reports in `hello`, its startup identity line and the journal header
  (`daemon`). A driver (`tools/preflight.sh`) refuses working-tree
  changes to the rung's own files (the tools, the control documents,
  the crates, the lock) and a running daemon, builds (`cargo build
  --workspace --locked`), writes
  `provenance.json` — HEAD, tree state, version, contract, the SHA-256
  of `acq` and of `acqd`, toolchain — into the run directory before any
  wire phase, and after the run holds every journal header to the
  `acqd` hash; by hand, a clean tree, `cargo build --workspace`, then
  run, and the ledger row names HEAD. Never rebuild `target/debug/acq`
  or `acqd` under a live daemon without `acq daemon stop` first (rung 8
  ran 34 h on a binary that predated the fix it was restarted to pick
  up); a rebuilt `acqd` under a live daemon is an artifact mismatch the
  next job command resolves by respawning. Reworded 2026-09-10 for the
  two identities; owner-approved 2026-09-11 (two clauses made exact
  after review the same day: the digest's inputs, the dirty check's
  scope).

Every job kind has had first contact as of 2026-08-30 (`RUN-LEDGER.md`:
`profile`, `leagues`, `character`; `leagues` was routed to `/league` until
that day), and the two poe2 character routes had theirs on 2026-09-02
(N41: poe2-suffixed policy names, free HEAD). A new kind gets the same
treatment under this rule. What the samples found that headers could not
teach (`/profile`: no rate headers, HEAD 403; `/account/leagues`: HEAD
counted) is carried as declared route knowledge (C32; ground truth
N38/N39, Q12).

## Rails

Knobs and defaults are in the README ("Knobs"); `acq daemon
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
   `RUN-LEDGER.md` is not a rail failure.
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
   had first contact on 2026-08-30 under the standing rule.
7. **Keyring save failure is surfaced** in `daemon status` and is a stop
   condition (permanent); the in-memory token is kept until exit.
8. **`ACQ_IDLE_SHUTDOWN=<secs>`** (permanent knob), set per rung.

Each rail has a deterministic test against the mock with rails 1 and 5
forced on; the suite passes unchanged with them off; quality gates from
`NETWORK-CLEANUP.md` stay green.

## Ladder (closed 2026-08-27)

Ten rungs from the first token exchange to a full `pull`, run 2026-08-22
to 2026-08-27 under a written hypothesis each; `RUN-LEDGER.md` holds
every execution. The goal was met with evidence: the daemon **halts
rather than floods** — ~1,450 live sends, **zero 429s**, one transient
origin 503 (N35) handled without a retry, R8 (the sleep-frozen expiry
clock) seen fixed live, and the rails proven on three real incidents:
the 503, a ceiling derived from another rung's count (rung 10; now
"derive from the run's own listing"), and a keyring-blind daemon spawned
from cron (the rung 8 re-soak; now `ACQ_NO_SPAWN=1` for cron). Rung 9
(timing-bucket measurement) is deferred on purpose: each attempt is a
counted violation, and rung 10's twenty holds bound the bucket well
enough. What the ladder taught lives where it applies — the standing
rule, the rails, the soak scripts' headers (retired 2026-09-09; in git
at `944406ff`), the ground-truth claims,
and the live-run skill (sleep and wakes). The rung table, the
preconditions, the soak mechanics and the three postmortems are at
`944406ff`. No further rungs are planned; live contact follows the
standing rule. Rung 11 (2026-08-30) was the one addition after closing,
written as a hypothesis because it asked a new question of GGG.
