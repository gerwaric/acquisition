#!/bin/sh
# Rung 8 soak: one `acq characters` per invocation (cron every 10 min).
# The rails env is set here too, so a respawned daemon keeps them.
set -u
DIR="$(cd "$(dirname "$0")/.." && pwd)"
export ACQ_GGG=1 ACQ_TRIPWIRE=1 ACQ_MAX_SENDS=200 ACQ_IDLE_SHUTDOWN=604800
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
OUT="$("$DIR/target/debug/acq" characters --json 2>&1)"
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
