---
name: mock-session
description: Work against the mock daemon in an isolated session — store (its socket derives from it), keyring — with a scripted login. Use before running acq for anything that is not a live run.
---

# Mock session

Everything here talks to the in-process mock provider; nothing reaches
GGG. Isolation exists so a session never shares a world, its socket, or
the persisted job queue with the owner's real daemon or with another
session — a world is one daemon's (its lock refuses a second, C83), and
its socket is derived from it, so two worlds are two rendezvous and
nothing has to be named by hand.

## Set up

```sh
unset ACQ_GGG ACQ_TRIPWIRE ACQ_MAX_SENDS ACQ_IDLE_SHUTDOWN   # a live run may have left these exported
export ACQ_NO_KEYRING=1                                       # sessions in memory only
export ACQ_STORE_DIR=<scratch>/store                          # the session's world (C83): never the real per-user data directory; its socket derives from it (`acq daemon status` prints it)
export ACQ_LOG_DIR=<scratch>/logs                             # the daemon log and journal, out of ~/Library/Logs
cargo build --workspace && ./target/debug/acq --version       # builds acq and the daemon acqd beside it (C82); after a cargo test, build again — it rewrites acq and acq-mcp
alias acq=./target/debug/acq
acq daemon status                                             # a fresh socket: "daemon is not running" (status never spawns, C10); after the login below it says provider mock
```

## Log in without a browser

```sh
acq auth --no-browser --json &          # prints {"authorize_url": …}
curl -sL "<that url with /authorize? replaced by /approve?>"     # add &user=NAME for a second account
```

`-L` is required: approve 302-redirects to the daemon's callback, and
without following it the login never completes (the login's own
profile job must land the account uuid). Any username is accepted;
policies count per username, so two accounts are one login apart.

## Rehearse a rung

`tools/tracer-rung.sh --mock ...` and `tools/persist-check.sh --mock ...`
run the identical flow the live runs use, exact ceilings included;
tracer evidence goes to `runs/mock/`.

## Tear down

`acq daemon stop`. Mock access tokens live 60 s, so a daemon left up
exercises silent refresh; it idles out on its own after 60 s with no
connections and no live jobs.
