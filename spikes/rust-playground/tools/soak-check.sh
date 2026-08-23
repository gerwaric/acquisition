#!/bin/sh
# Rung 8 stop-condition check over the journal, the run log, and daemon status.
set -u
DIR="$(cd "$(dirname "$0")/.." && pwd)"
T="$(python3 -c 'import tempfile;print(tempfile.gettempdir())')"
J="$T/acquisition-playground.ggg.sends.jsonl"
echo "== daemon status"; ACQ_GGG=1 "$DIR/target/debug/acq" daemon status 2>&1 | grep -E 'daemon|rails|HALT|KEYRING|REFRESH|not running'
echo "== runs.log: $(wc -l < "$DIR/runs/soak/runs.log" 2>/dev/null || echo 0) runs; non-success:"
grep -v ' success ' "$DIR/runs/soak/runs.log" 2>/dev/null || echo "  none"
echo "== journal since soak start ($1)"
python3 - "$J" "$1" <<'PY'
import json,sys,collections
path,since=sys.argv[1],sys.argv[2]
rows=[json.loads(l) for l in open(path) if l.strip()]
rows=[r for r in rows if r["ts"]>=since]
by=collections.Counter((r["method"],r["path"]) for r in rows)
for k,v in sorted(by.items()): print(f"  {v:5d}  {k[0]} {k[1]}")
bad=[r for r in rows if r["status"]!=200 and not (r["method"]=="HEAD" and r["status"]==204)]
print("  non-2xx/errors:", len(bad))
for r in bad[:10]: print("   ", r["ts"], r["method"], r["path"], r["status"], r["error"])
heads=collections.Counter((r["ts"][:10], r["path"]) for r in rows if r["method"]=="HEAD")
over=[(k,v) for k,v in heads.items() if v>1]
print("  HEADs per route per day >1:", over or "none")
posts=[r for r in rows if r["method"]=="POST"]
print("  token POSTs:", len(posts), [r["ts"] for r in posts][-5:])
PY
