---
name: live-run
description: Run the Rust daemon against the real GGG API — a first contact on a new endpoint, or the refresh loop under the rails. Human-run, from a terminal. Use before any command that talks to the real provider — the default since C88.
---

# Live run

The rule is `LIVE-TESTING.md` ("Standing rule: first contact"); this is
the procedure that follows it. Read the rule first. Nothing here talks to
GGG from an agent's shell: the real provider is the default (C88), and a
real-mode daemon is started only from a terminal — the CLI refuses to
spawn one without a terminal on stderr, and a daemon spawned from cron
or a background shell would have no keychain and no session anyway. A
live run is the owner's, from a terminal, under this procedure.

## Before

1. `unset ACQ_GGG ACQ_PROVIDER ACQ_TRIPWIRE ACQ_MAX_SENDS ACQ_IDLE_SHUTDOWN ACQ_STORE_DIR ACQ_LOG_DIR ACQ_NO_KEYRING`
   — the shell you were launched from may still export a previous run's
   rails or a mock session's isolation (`ACQ_PROVIDER=mock`), and a
   leftover `ACQ_GGG` is refused by every binary (C88).
2. `acq daemon stop`, until it says "daemon is not running". Never
   rebuild under a live daemon.
3. `git status --porcelain` must print nothing — the standing rule's
   clean tree for a hand run, the whole of it; the drivers check the
   rung's own files, `tools/preflight.sh`'s list — then `cargo build
   --workspace` (acq and the daemon
   `acqd` beside it, C82), then `./target/debug/acq version`
   — the binaries carry no commit: the contract revision and, for
   `acqd`, its file's hash (C84), so a dirty build's identity would be
   paired with a HEAD it is not; the journal header carries both and
   the ledger row names HEAD (`cargo test` never writes `acqd` and
   rewrites `acq-mcp` in its all-targets form; the plain build restores
   it — the drivers build first), then `tools/sign-acqd.sh` — the
   daemon's code identity, from `ACQ_CODESIGN_IDENTITY` in your shell
   profile; without it macOS Keychain asks twice per login after every
   rebuild. Sign after the build: one that re-links `acqd` writes a fresh
   unsigned one. The drivers do all of this themselves
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
ACQ_TRIPWIRE=1 ACQ_MAX_SENDS=3 acq <command>   # the real provider is the default (C88); no flag selects it
```

The first live run after C88 (2026-09-12) is also its proof, owed to
the ledger: with no provider flag set, the journal's `open` line and
`acq daemon status` name provider `ggg`, the world is the owner's data
directory and the keyring entry is the real service's — say so in the
row.

Read the journal before anything else (`acq daemon status` prints its
path: the world's `sends.jsonl` under the log directory, bounded; a
driver's daemons write the run directory's `sends.jsonl` instead, under
`ACQ_JOURNAL`): the probe line must report 0 hits on its policy. Hits > 0
means something else is on this account — stop and find it.

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
`acq jobs` (it reads the queue on disk once the halted daemon has idled
out, C45), cancel what should not go out, then respawn. An interrupted
driver leaves its `apply` parent persisted too: `acq cancel <id>` before
the driver runs again, or the parent's remaining children go out during
the next quote and the driver refuses the run (2026-09-08).

Known costs, not stops: every lifetime starts with a token POST (the
access token is never persisted), so a by-hand ceiling of 2 is one POST
and one GET; the first probe of a lifetime queues a few seconds behind
that POST; an unsigned `acqd` is asked twice per login after every
rebuild. The signed one (the 2026-09-12 row) is asked once per keychain
item on its first start under the identity — one prompt per persisted
account, "Always Allow" — and never again: not on the token write, not
after a rebuild. That first start waits at the prompts past the
client's 5 s handshake deadline (C86), so the client reports the daemon
unresponsive and names its pid for `kill` while it is only waiting on
you. Do not kill it: answer the prompts, then run the command again —
the daemon is up, or has idled out and its next start is silent.

Sleep and wakes (soaks): a closed laptop on AC dark-wakes every 15–60
min (Power Nap) and cron runs during the wakes; on battery it sleeps for
hours — `pmset -g log` is the evidence for any sleep claim. The first
request after a wake can fail in transport before the network is up;
rail 3 paces it as counted, and a consumer sees one failed job.
