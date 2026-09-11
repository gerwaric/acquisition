---
name: live-run
description: Run the Rust daemon against the real GGG API — a first contact on a new endpoint, or the refresh loop under the rails. Human-run, from a terminal. Use before any command that sets ACQ_GGG=1.
---

# Live run

The rule is `LIVE-TESTING.md` ("Standing rule: first contact"); this is
the procedure that follows it. Read the rule first. Nothing here talks to
GGG unless `ACQ_GGG=1` is set, and it is never set outside this
procedure. A live run is the owner's, from a terminal: a daemon spawned
from cron or a background shell has no keychain and no session.

## Before

1. `unset ACQ_GGG ACQ_TRIPWIRE ACQ_MAX_SENDS ACQ_IDLE_SHUTDOWN ACQ_SOCKET ACQ_STORE_DIR ACQ_NO_KEYRING`
   — the shell you were launched from may still export a previous run's
   rails or a mock session's isolation.
2. `acq daemon stop`. Never rebuild under a live daemon.
3. `git status --porcelain -- crates Cargo.toml Cargo.lock tools` must
   print nothing, then `cargo build --workspace` (acq and the daemon
   `acqd` beside it, C82), then `./target/debug/acq version`
   — the binaries carry no commit: the contract revision and, for
   `acqd`, its file's hash (C84), so a dirty build's identity would be
   paired with a HEAD it is not; the journal header carries both and
   the ledger row names HEAD (`cargo test` rewrites `acq` and `acq-mcp`
   with their all-targets forms and leaves `acqd`; the plain build
   restores them — the drivers build first). The drivers do all of this themselves
   (`tools/preflight.sh`, which hashes both executables into
   `provenance.json` and holds the journal to the `acqd` hash), and
   start `acqd` directly.
4. `ls -t runs/ | head` against `RUN-LEDGER.md`: know which run
   directories the ledger already cites before adding one — a bundle
   newer than the last row is a run that happened off the record.

## The refresh loop (policy → plan → apply → replan)

```sh
tools/tracer-rung.sh --account <SEL> [--realm R] [--characters all|id,...] <tab1,...|all>
```

The driver sets the rails, derives each cycle's ceiling from the plan
exactly, gates every wire phase on an explicit enter, verifies the
journal (`tools/tracer-verify.py`), copies the evidence to
`runs/<date>-tracer/`, and drafts the ledger row. It refuses a stale
binary, working-tree changes to its own files, leftover env, and a
running daemon. Rehearse first with `--mock` (evidence goes to
`runs/mock/`). Expect holds: none while a cycle stays under 15 stash
GETs, ~15 s before the 16th, ~343 s at most above 30 — the limiter
working, not a hang; macOS has no `timeout`.

## A first contact on a new endpoint

Fresh daemon, rails on, ceiling 3 (token POST, HEAD probe, GET; 2 if
the daemon already holds a valid access token):

```sh
ACQ_GGG=1 ACQ_TRIPWIRE=1 ACQ_MAX_SENDS=3 acq <command>
```

Read the journal before anything else (`acq daemon status` prints its
path): the probe line must report 0 hits on its policy. Hits > 0 means
something else is on this account — stop and find it.

## After

- The row in `RUN-LEDGER.md` (the driver's draft, or one written by hand
  for a first contact): date, tip, result, sends as POST/HEAD/GET,
  violations, the observed policy, the run directory, and where each
  lesson landed — never the lesson's text.
- A new fact about GGG is a numbered ground-truth claim, authored on the
  master-side branch and cherry-picked here — never the reverse.
- The owner's verdict is recorded **verbatim from the conversation**,
  marked as such; the driver's prompts are optional.
- `acq daemon stop`. Evidence stays in `runs/` (gitignored); the ledger
  cites the directory.

## If something trips

A tripwire trip is evidence to read, not a retry prompt. Write the cause
in the ledger, wait at least 360 s, then `acq daemon reset-tripwire`.
Never reset-and-retry. A ceiling halt leaves the queue waiting on disk:
`acq jobs`, cancel what should not go out, then respawn. An interrupted
driver leaves its `apply` parent persisted too: `acq cancel <id>` before
the driver runs again, or the parent's remaining children go out during
the next quote and the driver refuses the run (2026-09-08).

Known costs, not stops: the unsigned debug binary makes macOS Keychain
prompt twice per login after every rebuild; the first probe of a
lifetime queues a few seconds behind the token POST.

Sleep and wakes (soaks): a closed laptop on AC dark-wakes every 15–60
min (Power Nap) and cron runs during the wakes; on battery it sleeps for
hours — `pmset -g log` is the evidence for any sleep claim. The first
request after a wake can fail in transport before the network is up;
rail 3 paces it as counted, and a consumer sees one failed job.
