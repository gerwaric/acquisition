#!/usr/bin/env python3
"""call.py — run one surface call, save its stdout under raw/calls/, append one row to data/calls.csv.

usage:
  call.py --phase 1 --surface cli --q Q1 --into context [--note '...'] -- <argv...>
  call.py --phase 1 --surface mcp --q Q1 --into context [--note '...'] --tool search_items --args '{"text":"..."}'
  call.py ... --surface sql --db <path> --sql '<statement>'       (phase two only; opens read-only)
  call.py ... --surface workaround -- <argv...>                   (jq/python over a saved file; still a cost)

--into context prints the whole stdout (that IS the seat's cost); --into file prints a summary and the head.
rows: a JSON array's length; a JSON object's first list-valued field's length (else 1); otherwise non-empty lines.
"""
import argparse, csv, json, os, subprocess, sys, time, sqlite3

HERE = os.path.dirname(os.path.abspath(__file__))
TRACK = os.path.dirname(HERE)
CSV = os.path.join(TRACK, "data", "calls.csv")
OUT = os.path.join(TRACK, "raw", "calls")
FIELDS = ["n", "phase", "surface", "question", "verb_or_tool", "args", "exit", "rows", "bytes", "wall_ms", "into", "note"]

def count_rows(text):
    t = text.strip()
    if not t:
        return 0
    try:
        v = json.loads(t)
    except Exception:
        return sum(1 for line in t.splitlines() if line.strip())
    if isinstance(v, list):
        return len(v)
    if isinstance(v, dict):
        for k, x in v.items():
            if isinstance(x, list):
                return len(x)
        return 1
    return 1

def mcp_call(tool, args, timeout=600):
    exe = os.path.join(TRACK, "..", "..", "target", "debug", "acq-mcp")
    p = subprocess.Popen([exe], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    def send(o):
        p.stdin.write(json.dumps(o) + "\n"); p.stdin.flush()
    send({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "call.py", "version": "0"}}})
    p.stdout.readline()
    send({"jsonrpc": "2.0", "method": "notifications/initialized"})
    t0 = time.perf_counter()
    send({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": tool, "arguments": args}})
    line = p.stdout.readline()
    wall = (time.perf_counter() - t0) * 1000
    p.stdin.close(); p.wait(timeout=30)
    resp = json.loads(line)
    if "error" in resp:
        return 1, json.dumps(resp["error"]), wall
    r = resp["result"]
    text = "".join(c.get("text", "") for c in r.get("content", []))
    return (1 if r.get("isError") else 0), text, wall

def sql_call(db, sql):
    uri = "file:" + db + "?mode=ro"
    con = sqlite3.connect(uri, uri=True)
    con.execute("PRAGMA query_only=1")
    t0 = time.perf_counter()
    try:
        cur = con.execute(sql)
        rows = cur.fetchall()
        wall = (time.perf_counter() - t0) * 1000
        hdr = [d[0] for d in cur.description] if cur.description else []
        out = "\n".join(["|".join(hdr)] + ["|".join("" if c is None else str(c) for c in r) for r in rows]) + "\n"
        return 0, out, wall, len(rows)
    except Exception as e:
        wall = (time.perf_counter() - t0) * 1000
        return 1, f"error: {e}\n", wall, 0
    finally:
        con.close()

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--phase", required=True); ap.add_argument("--surface", required=True)
    ap.add_argument("--q", required=True); ap.add_argument("--into", required=True, choices=["context", "file"])
    ap.add_argument("--note", default=""); ap.add_argument("--tool"); ap.add_argument("--args", default="{}")
    ap.add_argument("--db"); ap.add_argument("--sql"); ap.add_argument("--head", type=int, default=20)
    ap.add_argument("argv", nargs="*")
    a = ap.parse_args()
    os.makedirs(OUT, exist_ok=True); os.makedirs(os.path.dirname(CSV), exist_ok=True)
    new = not os.path.exists(CSV)
    n = 1
    if not new:
        with open(CSV, newline="") as f:
            n = sum(1 for r in csv.reader(f) if not (r and r[0].startswith("#")))  # header counts as 1 → next n; the # provenance line does not
    rows_override = None
    if a.surface == "mcp":
        verb, args = a.tool, a.args
        code, text, wall = mcp_call(a.tool, json.loads(a.args))
    elif a.surface == "sql" and a.sql is not None:
        verb, args = os.path.basename(a.db), a.sql
        code, text, wall, rows_override = sql_call(a.db, a.sql)
    else:
        argv = a.argv
        verb = " ".join(os.path.basename(argv[0:1][0] if argv else "") for _ in [0]) + (" " + " ".join(x for x in argv[1:] if not x.startswith("{") and len(x) < 40) if len(argv) > 1 else "")
        args = " ".join(argv[1:])
        t0 = time.perf_counter()
        r = subprocess.run(argv, capture_output=True, text=True)
        wall = (time.perf_counter() - t0) * 1000
        code, text = r.returncode, r.stdout + (("\n[stderr] " + r.stderr) if r.stderr.strip() else "")
    b = len(text.encode())
    rows = rows_override if rows_override is not None else count_rows(text)
    path = os.path.join(OUT, f"{n:03d}.out")
    with open(path, "w") as f:
        f.write(text)
    with open(CSV, "a", newline="") as f:
        w = csv.DictWriter(f, fieldnames=FIELDS, lineterminator="\n")
        if new:
            w.writeheader()
        w.writerow({"n": n, "phase": a.phase, "surface": a.surface, "question": a.q, "verb_or_tool": verb, "args": args,
                    "exit": code, "rows": rows, "bytes": b, "wall_ms": round(wall), "into": a.into, "note": a.note})
    print(f"[call {n:03d}] {a.surface} {verb} | exit {code} | rows {rows} | {b} B | {wall:.0f} ms | -> {a.into} | {path}")
    if a.into == "context":
        sys.stdout.write(text if text.endswith("\n") else text + "\n")
    else:
        head = "\n".join(text.splitlines()[: a.head])
        cap = a.head * 160
        if len(head) > cap:
            head = head[:cap] + " …[head cut at %d chars]" % cap
        sys.stdout.write(head + ("\n…\n" if len(text.splitlines()) > a.head else "\n"))

if __name__ == "__main__":
    main()
