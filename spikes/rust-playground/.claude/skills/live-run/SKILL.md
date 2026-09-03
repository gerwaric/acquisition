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
3. `cargo build`, then verify the **binary**, not the checkout:
   `./target/debug/acq --version` must equal `git rev-parse --short=12 HEAD`
   with no `-dirty` (`cargo test` and `cargo clippy` do not rebuild
   `target/debug/acq`).
4. `ls -t runs/ | head` against the run ledger: know which run
   directories the ledger already cites before adding one.

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

- The ledger row (the driver's draft, or one written by hand for a
  first contact): date, tip, result, sends as POST/HEAD/GET, violations,
  the observed policy.
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
`acq jobs`, cancel what should not go out, then respawn.

Known costs, not stops: the unsigned debug binary makes macOS Keychain
prompt twice per login after every rebuild; the first probe of a
lifetime queues a few seconds behind the token POST.
