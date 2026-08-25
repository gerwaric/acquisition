#!/bin/sh
# Rung 8 stop-condition check over the journal, the run log, and daemon status.
set -u
if [ $# -lt 1 ]; then
    echo "usage: $(basename "$0") <since>" >&2
    echo "  <since> may be partial: 2026, 2026-08, 2026-08-24," >&2
    echo "  2026-08-24T10, 2026-08-24T10:30, 2026-08-24T10:30:14[.807][Z]" >&2
    echo "  Missing components read as the earliest instant (UTC)." >&2
    exit 2
fi
DIR="$(cd "$(dirname "$0")/.." && pwd)"
T="$(python3 -c 'import tempfile;print(tempfile.gettempdir())')"
J="$T/acquisition-playground.ggg.sends.jsonl"
echo "== daemon status"; ACQ_GGG=1 "$DIR/target/debug/acq" daemon status 2>&1 | grep -E 'daemon|rails|HALT|KEYRING|REFRESH|not running'
HEAD_REV="$(git -C "$DIR" rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
BIN_REV="$("$DIR/target/debug/acq" --version 2>/dev/null | sed -n 's/.*(\(.*\)).*/\1/p')"
echo "== provenance: checkout $HEAD_REV, binary $BIN_REV$( [ "$HEAD_REV" = "$BIN_REV" ] || echo '  ** MISMATCH: the binary is not the checkout **')"
echo "== runs.log: $(wc -l < "$DIR/runs/soak/runs.log" 2>/dev/null || echo 0) runs; non-success:"
grep -v ' success ' "$DIR/runs/soak/runs.log" 2>/dev/null || echo "  none"
python3 - "$J" "$1" "$BIN_REV" <<'PY'
import json,sys,collections
from datetime import datetime, timezone

def parse_ts(text, what):
    """Parse a full or partial UTC timestamp into a datetime.

    Journal stamps carry fractional seconds and a 'Z' ("...T01:30:14.807Z"),
    so `since` must never be compared to them as a string: '.' sorts below
    'Z', which silently dropped every send in the same second as the start.
    Missing components read as the earliest instant, making a partial like
    "2026-08-24" mean that day's midnight.
    """
    s = text.strip()
    if s[-1:] in ("Z", "z"):
        s = s[:-1]
    s = s.replace(" ", "T", 1)
    date, _, time = s.partition("T")
    try:
        d = [int(p) for p in date.split("-")]
        if not 1 <= len(d) <= 3:
            raise ValueError("date has too many parts")
        d += [1] * (3 - len(d))
        h = mi = 0
        sec = 0.0
        if time:
            t = time.split(":")
            if len(t) > 3:
                raise ValueError("time has too many parts")
            h = int(t[0])
            if len(t) > 1: mi = int(t[1])
            if len(t) > 2: sec = float(t[2])
        micro = round(sec % 1 * 1_000_000)
        return datetime(d[0], d[1], d[2], h, mi, int(sec), micro, tzinfo=timezone.utc)
    except ValueError as e:
        sys.exit(f"soak-check: cannot read {what} {text!r}: {e}")

path, since_text, bin_rev = sys.argv[1], sys.argv[2], sys.argv[3]
since = parse_ts(since_text, "timestamp")
print(f"== journal since {since.isoformat().replace('+00:00','Z')} (from {since_text!r})")
rows = [json.loads(l) for l in open(path) if l.strip()]
# One "open" line per daemon lifetime carries provenance (pid, build, clock).
# A manual-clock journal is a scenario, never live evidence.
opens = [r for r in rows if r.get("event") == "open"]
rows = [r for r in rows if "event" not in r]
for o in opens:
    if o.get("clock") != "system":
        sys.exit(f"soak-check: journal has a {o.get('clock')!r}-clock lifetime (pid {o.get('pid')}); not live evidence")
print("  daemon lifetimes:", [(o["pid"], o.get("build", "?")) for o in opens] or "none recorded (pre-header journal)")
rows = [r for r in rows if parse_ts(r["ts"], "journal stamp") >= since]
# Every lifetime inside the window must be the binary on disk: rung 8 ran
# 34 h on a build predating the fix it was restarted for.
live_pids = {r["pid"] for r in rows}
stale = [(o["pid"], o.get("build")) for o in opens if o["pid"] in live_pids and o.get("build") != bin_rev]
if stale:
    sys.exit(f"soak-check: lifetimes not built from the current binary ({bin_rev}): {stale}")
by = collections.Counter((r["method"], r["path"]) for r in rows)
for k, v in sorted(by.items()): print(f"  {v:5d}  {k[0]} {k[1]}")
print(f"  {len(rows):5d}  TOTAL sends")
bad = [r for r in rows if r["status"] != 200 and not (r["method"] == "HEAD" and r["status"] == 204)]
print("  non-2xx/errors:", len(bad))
for r in bad[:10]: print("   ", r["ts"], r["method"], r["path"], r["status"], r["error"])
# One probe per (daemon lifetime, route): a second HEAD on a route inside
# one pid is a HEAD stream (N2's shape); a HEAD after a restart is not.
heads = collections.Counter((r["pid"], r["route"]) for r in rows if r["method"] == "HEAD")
over = [(k, v) for k, v in heads.items() if v > 1]
print("  HEADs per (pid, route) >1:", over or "none")
print("  lifetimes in window:", sorted(live_pids))
posts = [r for r in rows if r["method"] == "POST"]
print("  token POSTs:", len(posts), [r["ts"] for r in posts][-5:])
PY
