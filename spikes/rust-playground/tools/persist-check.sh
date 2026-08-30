#!/bin/bash
# Persistence check (LIVE-TESTING.md, "Persistence check — halt, crash,
# resume"): drive the five steps, verify the expectations from the journal,
# and draft the ledger row. The human parts stay human: run this from a
# terminal (the daemon needs your keychain), and every wire phase waits for
# an explicit enter.
#
#   tools/persist-check.sh [--account NAME] <tab1,...,tab6>   # live, rails on
#   tools/persist-check.sh --mock <tab1,...,tab6>              # same flow, mock
#
# With more than one persisted account, every job needs a selector: pass
# --account (or have ACQ_ACCOUNT exported) naming the account whose tabs
# these are.
#
# Live mode sends ~13 real requests across two daemon lifetimes:
#   lifetime 1 (ceiling 6):  POST, HEAD+GET list, HEAD stash, ~2 child GETs,
#                            then the ceiling halts the queue
#   kill -9 the halted daemon
#   lifetime 2 (ceiling 10): POST, HEAD stash (probe reads the real counters
#                            — its hits > 0 are OURS, expected), rest of the
#                            children; the parent finishes done
# Mock mode proves the script itself on the in-process provider first:
# isolated socket + store, a throwaway mock login (the mock's keyring
# service is separate from the real one; NOT ACQ_NO_KEYRING — the session
# must survive the kill), logout at the end. One mock-mode caveat: the mock
# provider dies with the daemon, so lifetime 2's probe reads fresh counters
# there — the probe carrying lifetime 1's hits is evidence only live, which
# is the wire premise this run exists to check.
set -euo pipefail

here=$(cd "$(dirname "$0")/.." && pwd)
ACQ="$here/target/debug/acq"

MODE=live
ACCOUNT=
while [ $# -gt 0 ]; do
    case "$1" in
    --mock) MODE=mock; shift ;;
    --account) ACCOUNT=${2:?--account needs a value}; shift 2 ;;
    *) break ;;
    esac
done
TABS=${1:?usage: persist-check.sh [--mock] [--account NAME] <tab1,...,tab6>}
ntabs=$(echo "$TABS" | tr ',' '\n' | grep -c . || true)
if [ "$ntabs" -ne 6 ]; then
    echo "need exactly 6 tab ids (got $ntabs) — the ceilings are derived for 6" >&2
    exit 2
fi
command -v jq >/dev/null || { echo "jq is required" >&2; exit 2; }
if [ "$MODE" = live ] && [ ! -t 0 ]; then
    echo "refusing: stdin is not a terminal — the wire phases gate on enter." >&2
    echo "run this directly from a terminal, not via a captured or piped shell." >&2
    exit 2
fi

# ---- preflight (no wire) ---------------------------------------------------

for v in ACQ_SOCKET ACQ_STORE_DIR ACQ_NO_KEYRING ACQ_NO_SPAWN ACQ_JOURNAL; do
    if [ -n "${!v:-}" ]; then
        echo "refusing: $v is set in this shell (leftover from other work); unset it first" >&2
        exit 2
    fi
done
unset ACQ_GGG ACQ_TRIPWIRE ACQ_MAX_SENDS ACQ_IDLE_SHUTDOWN

tip=$(git -C "$here" rev-parse --short=12 HEAD)
ver=$("$ACQ" --version)
case "$ver" in
*"$tip"*-dirty* | *dirty*)
    echo "refusing: binary is dirty ($ver) — commit, cargo build, retry" >&2
    [ "$MODE" = mock ] && echo "(mock mode: continuing anyway)" || exit 2 ;;
*"$tip"*) ;;
*)
    echo "refusing: binary ($ver) does not match HEAD ($tip) — cargo build first" >&2
    exit 2 ;;
esac

RUN_DIR="$here/runs/$(date -u +%F)-persist"
if [ "$MODE" = mock ]; then RUN_DIR="$RUN_DIR-mock"; fi
mkdir -p "$RUN_DIR"

if [ "$MODE" = live ]; then
    export ACQ_GGG=1
    T=${TMPDIR:-/tmp}; T=${T%/}
    SOCK="$T/acquisition-playground.sock"
    PROVIDER=ggg
else
    export ACQ_SOCKET=/tmp/acq-persist.sock ACQ_STORE_DIR="$RUN_DIR/store"
    SOCK=$ACQ_SOCKET
    PROVIDER=mock
fi
# The only daemons in this run are the ones this script starts, with the
# rails it says; a client must never lazy-spawn (or replace) one.
export ACQ_NO_SPAWN=1
JOURNAL="${SOCK%.sock}.$PROVIDER.sends.jsonl"
LOG="${SOCK%.sock}.log"

status_json() { "$ACQ" daemon status --json 2>/dev/null || echo '{}'; }

if [ -S "$SOCK" ] && [ "$(status_json | jq -r '.pid // empty')" != "" ]; then
    echo "refusing: a daemon is already running on $SOCK — acq daemon stop first" >&2
    exit 2
fi

COMPLETED=0
REFRESH_PID=
cleanup() {
    if [ -n "$REFRESH_PID" ]; then kill "$REFRESH_PID" 2>/dev/null || true; fi
    if [ "$COMPLETED" != 1 ]; then
        echo ""
        if [ -z "${PID1:-}" ]; then
            echo "*** aborted before any daemon started; nothing was sent."
        else
            echo "*** aborted mid-run. The queue is on disk; a live daemon may still hold it."
            echo "*** inspect with: acq jobs / acq daemon status; cancel what should not resume."
            echo "*** partial evidence is in $RUN_DIR and $JOURNAL"
        fi
    fi
    return 0
}
trap cleanup EXIT

spawn_daemon() { # <max_sends> <outfile>
    env ACQ_TRIPWIRE=1 ACQ_MAX_SENDS="$1" ACQ_IDLE_SHUTDOWN=600 \
        "$ACQ" daemon run >"$RUN_DIR/$2" 2>&1 &
    for _ in $(seq 1 100); do
        pid=$(status_json | jq -r '.pid // empty')
        [ -n "$pid" ] && { echo "$pid"; return 0; }
        sleep 0.1
    done
    echo "daemon did not come up; see $RUN_DIR/$2 and $LOG" >&2
    return 1
}

confirm() {
    echo ""
    echo ">>> $1"
    read -r -p ">>> enter to proceed (ctrl-c to abort) "
}

echo "persistence check ($MODE) — binary $ver"
echo "socket $SOCK | journal $JOURNAL | evidence -> $RUN_DIR"
if [ -n "$ACCOUNT" ]; then export ACQ_ACCOUNT="$ACCOUNT"; fi
if [ "$MODE" = live ]; then
    echo "accounts on this machine (the fresh daemon restores the persisted ones):"
    "$ACQ" accounts || true
    persisted=$("$ACQ" accounts --json 2>/dev/null |
        jq '[.[] | select(.persisted)] | length' || echo 0)
    if [ "$persisted" -gt 1 ] && [ -z "${ACQ_ACCOUNT:-}" ]; then
        echo "refusing: $persisted persisted accounts — every job needs a selector." >&2
        echo "rerun with --account <username>, the account whose tabs these are." >&2
        exit 2
    fi
    if [ -n "${ACQ_ACCOUNT:-}" ]; then echo "acting as: $ACQ_ACCOUNT"; fi
fi
OFFSET=0
if [ -f "$JOURNAL" ]; then OFFSET=$(wc -c <"$JOURNAL" | tr -d ' '); fi

# ---- lifetime 1: refresh into the ceiling halt -----------------------------

confirm "lifetime 1: daemon with ceiling 6, refresh of 6 tabs; ~6 sends then a halt"
PID1=$(spawn_daemon 6 daemon1.out)
echo "daemon 1: pid $PID1"

if [ "$MODE" = mock ]; then
    # Scripted mock login (AGENTS.md): auth --no-browser, then approve.
    "$ACQ" auth --no-browser >"$RUN_DIR/auth.out" 2>&1 &
    AUTH_PID=$!
    URL=
    for _ in $(seq 1 50); do
        URL=$(grep -o 'http://[^ ]*authorize[^ ]*' "$RUN_DIR/auth.out" | head -1 || true)
        [ -n "$URL" ] && break
        sleep 0.1
    done
    [ -n "$URL" ] || { echo "no login URL in auth.out" >&2; exit 1; }
    APPROVE=$(echo "$URL" | sed 's|/authorize?|/approve?|')
    curl -sL "$APPROVE&user=persist-check" >/dev/null
    wait "$AUTH_PID"
fi

"$ACQ" refresh --tabs "$TABS" >"$RUN_DIR/refresh.out" 2>&1 &
REFRESH_PID=$!

PARENT=
HALTED=
for _ in $(seq 1 240); do
    s=$(status_json)
    [ -z "$PARENT" ] && PARENT=$("$ACQ" jobs --json 2>/dev/null |
        jq -r '[.[] | select(.kind == "refresh")][0].id // empty' || true)
    HALTED=$(echo "$s" | jq -r '.rails.halted // empty')
    [ -n "$HALTED" ] && break
    if ! kill -0 "$REFRESH_PID" 2>/dev/null; then
        echo "refresh client exited before any halt — its output:" >&2
        cat "$RUN_DIR/refresh.out" >&2
        exit 1
    fi
    sleep 0.5
done
[ -n "$HALTED" ] || { echo "no halt within 120s; read $JOURNAL" >&2; exit 1; }
case "$HALTED" in
*ceiling*) echo "halted as planned: $HALTED" ;;
*)
    echo "*** TRIPWIRE TRIP, not the ceiling: $HALTED" >&2
    echo "*** stop. Read the journal; observe the 360s post-violation rule." >&2
    exit 1 ;;
esac
[ -n "$PARENT" ] || { echo "never saw the refresh parent in acq jobs" >&2; exit 1; }
sleep 2 # let an already-dispatched send land (the gate admits two)

# The client leaves; its jobs stay (decided 2026-08-24).
{ kill "$REFRESH_PID" && wait "$REFRESH_PID"; } 2>/dev/null || true
REFRESH_PID=

"$ACQ" jobs --json >"$RUN_DIR/jobs-halted.json"
failed=$(jq '[.[] | select(.state == "failed")] | length' "$RUN_DIR/jobs-halted.json")
waiting=$(jq '[.[] | select(.state == "waiting" and .kind == "stash")] | length' \
    "$RUN_DIR/jobs-halted.json")
if [ "$failed" != 0 ]; then
    echo "*** $failed job(s) FAILED under the halt (pre-persistence behavior):" >&2
    jq '[.[] | select(.state == "failed")]' "$RUN_DIR/jobs-halted.json" >&2
    exit 1
fi
echo "parent job $PARENT held; $waiting stash children waiting, none failed:"
"$ACQ" jobs

# ---- kill -9, restore, resume ----------------------------------------------

confirm "kill -9 daemon pid $PID1 (mid-halt: nothing is in flight); queue stays on disk"
kill -9 "$PID1"
for _ in $(seq 1 50); do kill -0 "$PID1" 2>/dev/null || break; sleep 0.1; done
kill -0 "$PID1" 2>/dev/null && { echo "pid $PID1 did not die" >&2; exit 1; }
echo "daemon 1 gone."

confirm "lifetime 2: fresh daemon, ceiling 10; it restores the queue, probes, finishes (~6 sends)"
PID2=$(spawn_daemon 10 daemon2.out)
echo "daemon 2: pid $PID2 — waiting for parent job $PARENT to finish"

STATE=
for _ in $(seq 1 400); do
    if ! kill -0 "$PID2" 2>/dev/null; then
        echo "*** daemon 2 died while the parent was still '${STATE:-unknown}':" >&2
        tail -5 "$RUN_DIR/daemon2.out" "$LOG" >&2 || true
        exit 1
    fi
    STATE=$("$ACQ" status "$PARENT" --json 2>/dev/null | jq -r '.state // empty' || true)
    case "$STATE" in done | failed | cancelled) break ;; esac
    sleep 1
done
case "$STATE" in
done) echo "parent $PARENT finished done across two daemon lifetimes." ;;
failed | cancelled)
    echo "*** parent finished '$STATE', not done — read the result and the journal:" >&2
    "$ACQ" result "$PARENT" >&2 || true
    exit 1 ;;
*)
    echo "*** parent still '$STATE' after 400s — a limiter hold this long is unexpected here" >&2
    exit 1 ;;
esac

"$ACQ" result "$PARENT" --json >"$RUN_DIR/parent-result.json"
unknown=$(jq -r '.payload.unknown_tab_ids // [] | join(",")' "$RUN_DIR/parent-result.json")
if [ -n "$unknown" ]; then
    echo "note: requested id(s) not in the account's tab list, never fetched: $unknown"
fi
if [ "$MODE" = mock ]; then "$ACQ" auth logout >/dev/null 2>&1 || true; fi
"$ACQ" daemon stop >/dev/null
# The wire phases are over and no daemon is left; what remains is analysis,
# so the trap's "a daemon may still hold the queue" warning no longer applies.
COMPLETED=1

# ---- evidence and verification ---------------------------------------------

cp "$JOURNAL" "$RUN_DIR/sends.jsonl"
cp "$LOG" "$RUN_DIR/daemon.log" 2>/dev/null || true

verify() { python3 - "$JOURNAL" "$OFFSET" <<'PY'
import json, sys

f = open(sys.argv[1]); f.seek(int(sys.argv[2]))
lifetimes, cur = [], None
for raw in f:
    if not raw.strip():
        continue
    l = json.loads(raw)
    if l.get("event") == "open":
        cur = {"pid": l["pid"], "build": l["build"], "clock": l["clock"], "sends": []}
        lifetimes.append(cur)
        continue
    if cur is None:
        cur = {"pid": l.get("pid"), "build": "?", "clock": "?", "sends": []}
        lifetimes.append(cur)
    cur["sends"].append(l)

fail, totals = [], []
for i, lt in enumerate(lifetimes, 1):
    print(f"lifetime {i}: pid {lt['pid']}  build {lt['build']}  clock {lt['clock']}")
    counts, first = {}, {}
    for s in lt["sends"]:
        m, r, st = s["method"], s["route"], s.get("status")
        counts[m] = counts.get(m, 0) + 1
        first.setdefault(r, m)
        flag = ""
        if s.get("error") or st is None or st >= 400:
            flag = "  <-- NOT OK"
            fail.append(f"lifetime {i}: {m} {r} -> {st} error={s.get('error')}")
        if st == 429:
            fail.append(f"lifetime {i}: 429 on {r}")
        if m == "HEAD":
            print(f"  HEAD {r} -> {st}  rate {json.dumps(s.get('rate'))}{flag}")
        else:
            print(f"  {m} {r} -> {st}  wait_ms {s.get('wait_ms')}{flag}")
    for r, m in first.items():
        if r != "oauth-token" and m != "HEAD":
            fail.append(f"lifetime {i}: first send on {r} was {m}, not the probe")
    t = f"{counts.get('POST',0)}/{counts.get('HEAD',0)}/{counts.get('GET',0)}"
    totals.append(f"{t} = {len(lt['sends'])}")
    print(f"  totals (POST/HEAD/GET): {totals[-1]}")

if len(lifetimes) != 2:
    fail.append(f"expected 2 daemon lifetimes in this run's journal, saw {len(lifetimes)}")

print()
if fail:
    print("CHECKS FAILED — a ledger row still gets written, saying what happened:")
    for x in fail:
        print(f"  - {x}")
    sys.exit(1)
print("checks passed: halt left nothing failed, every route probed before its")
print("first send in both lifetimes, no non-2xx. Read lifetime 2's HEAD line")
print("above: its rate state carrying lifetime 1's hits IS the restart-replay")
print("evidence (those hits are ours — the standing rule's 'stop and find it'")
print("does not apply to this run).")
print()
print("draft ledger row:")
print(f"| <date> | persist | <tip> | pass | L1 {totals[0]}, L2 {totals[1]} | 0 |"
      " ceiling halt left children waiting; kill -9 mid-halt; successor probed"
      " before resuming; parent done across lifetimes; runs/<date>-persist/ |")
PY
}
if ! verify | tee "$RUN_DIR/summary.txt"; then
    echo ""
    echo "*** verification FAILED — see above; evidence in $RUN_DIR" >&2
    exit 1
fi

echo ""
echo "evidence in $RUN_DIR (journal, daemon log, job snapshots, summary)."
if [ "$MODE" = live ]; then
    echo "next: paste the ledger row into LIVE-TESTING.md's run ledger."
fi
