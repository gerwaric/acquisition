#!/bin/sh
# Rung 8 soak: one `acq characters` per invocation (cron every 10 min).
# The rails env is set here too, so a respawned daemon keeps them.
#
# The send ceiling is derived, not guessed (LIVE-TESTING preconditions):
# cron cadence 6/h x 24 h = 144 GETs per day, plus one token POST per ~10 h
# and one HEAD per daemon lifetime, for SOAK_DAYS days (default 7). Rung 8
# ran with a flat 200 and was cut off at 33 h of a "several days" intent.
#
# Cron never spawns the daemon: a daemon started from cron has no keychain
# access on macOS and comes up with no session (2026-08-25, first tick of the
# re-soak: "not logged in", zero sends). ACQ_NO_SPAWN makes the CLI refuse
# instead. The daemon is started from a terminal, once, by a person:
#   ACQ_GGG=1 ACQ_TRIPWIRE=1 ACQ_MAX_SENDS=<ceiling> ACQ_IDLE_SHUTDOWN=604800 \
#       runs/soak/acq characters
# and the same after any deliberate `daemon stop`.
#
# The soak runs a frozen copy of the binary, `runs/soak/acq`, so the tree
# can be rebuilt while the daemon lives (the precondition forbids rebuilding
# the binary a live daemon was spawned from; the version handshake cannot
# see it). Freeze it at the start: `cp target/debug/acq runs/soak/acq`.
set -u
DIR="$(cd "$(dirname "$0")/.." && pwd)"
ACQ="$DIR/runs/soak/acq"
SOAK_DAYS="${SOAK_DAYS:-2}"
CEILING=$(( SOAK_DAYS * 144 + SOAK_DAYS * 3 + 10 ))
export ACQ_GGG=1 ACQ_TRIPWIRE=1 ACQ_MAX_SENDS="$CEILING" ACQ_IDLE_SHUTDOWN=604800 ACQ_NO_SPAWN=1
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if [ ! -x "$ACQ" ]; then
    printf '%s rc=127 NOBINARY %s\n' "$TS" "$ACQ" >> "$DIR/runs/soak/runs.log"
    exit 127
fi
OUT="$("$ACQ" characters --json 2>&1)"
RC=$?
LINE="$(printf '%s' "$OUT" | python3 -c '
import json,sys
raw=sys.stdin.read()
try:
    d=json.loads(raw); p=d.get("payload",{})
    print(d.get("outcome"), p.get("rate_limit",{}).get("x-rate-limit-account-state"))
except Exception:
    print("NONJSON", raw.replace("\n"," ")[:300])')"
printf '%s rc=%s %s\n' "$TS" "$RC" "$LINE" >> "$DIR/runs/soak/runs.log"
# Anything but a clean success gets a desktop notification (macOS), so a
# tripwire trip or a failing daemon is noticed without polling the log.
case "$LINE" in
    success*) ;;
    *) osascript -e "display notification \"$(printf '%s' "$LINE" | cut -c1-200 | tr '"' "'")\" with title \"acq soak: run not successful\"" >/dev/null 2>&1 ;;
esac
