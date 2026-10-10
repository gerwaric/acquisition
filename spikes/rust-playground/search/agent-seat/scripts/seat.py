#!/usr/bin/env python3
"""seat.py — the MCP seat's one door (step 11c): one call to `acq-mcp` over stdio, logged.

usage:
  seat.py open                                  what a host shows before any call: the server's instructions and its tools
  seat.py call --q Q1 <tool> '<json arguments>' [--note '...']
  seat.py call --q Q1 <tool> - [--note '...']   the arguments read from stdin

Every call is a row of data/mcp-seat-calls.csv and a file under raw/seat-11c/calls/ (gitignored).
The world is set here and nowhere else: a fresh `.backup` of the 2026-09-14 copy (MANIFEST.md),
the mock provider, no daemon. One server process per call; a request is whole, so nothing is held between.

bytes: `text_bytes` is the result's text block, what 2026-09-14's `bytes` counted; `result_bytes` is the
whole JSON-RPC line, which carries the document a second time as structured content.
A result is printed whole up to SHOWN bytes of text; past that its head is printed and the row says `head`.
REFUSED tools write the store or reach for a daemon: the door refuses them and logs the attempt.
"""
import argparse, csv, json, os, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
TRACK = os.path.dirname(HERE)
ROOT = os.path.dirname(os.path.dirname(TRACK))
RUN = os.path.join(TRACK, "raw", "seat-11c")
EXE = os.path.join(ROOT, "target", "release", "acq-mcp")
CSV = os.path.join(TRACK, "data", "mcp-seat-calls.csv")
OUT = os.path.join(RUN, "calls")
HEADER = "# written by scripts/seat.py during the run of BRIEF.md, one row per call to acq-mcp over the 2026-09-14 copy; row n's text is raw/seat-11c/calls/NNN.json (gitignored)"
FIELDS = ["n", "question", "tool", "args", "is_error", "text_bytes", "result_bytes", "wall_ms", "shown", "note"]
SHOWN = 150_000
HEAD = 2_000
REFUSED = {"set_sync_policy", "apply_plan", "submit_job", "cancel_job"}
DROPPED = ("ACQ_GGG", "ACQ_TRIPWIRE", "ACQ_MAX_SENDS", "ACQ_IDLE_SHUTDOWN", "ACQ_ACCOUNT")


def server():
    env = {k: v for k, v in os.environ.items() if k not in DROPPED}
    env.update(ACQ_PROVIDER="mock", ACQ_NO_KEYRING="1", ACQ_NO_SPAWN="1",
               ACQ_STORE_DIR=os.path.join(RUN, "store"), ACQ_LOG_DIR=os.path.join(RUN, "logs"))
    p = subprocess.Popen([EXE], stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, env=env)

    def rpc(o):
        p.stdin.write(json.dumps(o) + "\n")
        p.stdin.flush()
        if "id" not in o:
            return None, 0.0
        t0 = time.perf_counter()
        line = p.stdout.readline()
        return line, (time.perf_counter() - t0) * 1000

    first, _ = rpc({"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
        "protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "seat.py", "version": "0"}}})
    rpc({"jsonrpc": "2.0", "method": "notifications/initialized"})
    return p, rpc, first


def close(p):
    p.stdin.close()
    p.wait(timeout=30)


def log(question, tool, args, is_error, text, result_bytes, wall, shown, note):
    os.makedirs(OUT, exist_ok=True)
    new = not os.path.exists(CSV)
    n = 1
    if not new:
        with open(CSV, newline="") as f:
            n = sum(1 for r in csv.reader(f) if r and not r[0].startswith("#"))  # the header row counts as 1, so this is the next n
    path = os.path.join(OUT, f"{n:03d}.json")
    with open(path, "w") as f:
        f.write(text)
    with open(CSV, "a", newline="") as f:
        if new:
            f.write(HEADER + "\n")
        w = csv.DictWriter(f, fieldnames=FIELDS, lineterminator="\n")
        if new:
            w.writeheader()
        w.writerow({"n": n, "question": question, "tool": tool, "args": args, "is_error": int(is_error),
                    "text_bytes": len(text.encode()), "result_bytes": result_bytes, "wall_ms": round(wall),
                    "shown": shown, "note": note})
    return n


def show(n, tool, is_error, text, result_bytes, wall, shown):
    print(f"[call {n:03d}] {tool} | {'refused' if is_error else 'answered'} | {len(text.encode())} B of text, "
          f"{result_bytes} B on the wire | {wall:.0f} ms | shown {shown}")
    if shown == "whole":
        print(text)
    else:
        print(text[:HEAD])
        print(f"… [the door: this result is {len(text.encode())} B, past the {SHOWN} B it shows whole; only its first {HEAD} characters are above]")


def open_cmd():
    p, rpc, first = server()
    line, wall = rpc({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})
    close(p)
    init = json.loads(first)["result"]
    tools = json.loads(line)["result"]["tools"]
    text = json.dumps({"instructions": init.get("instructions", ""), "tools": tools}, ensure_ascii=False, separators=(",", ":"))
    n = log("open", "initialize + tools/list", "", False, text, len(first.encode()) + len(line.encode()), wall, "whole", "")
    show(n, "initialize + tools/list", False, text, len(first.encode()) + len(line.encode()), wall, "whole")


def call_cmd(a):
    raw = sys.stdin.read() if a.args == "-" else a.args
    try:
        args = json.loads(raw)
    except ValueError as e:
        sys.exit(f"seat.py: the arguments are not JSON ({e}); nothing was sent and nothing logged")
    sent = json.dumps(args, separators=(",", ":"))
    if a.tool in REFUSED:
        text = f"the door refuses `{a.tool}`: it writes the store or reaches for a daemon, and this seat reads"
        n = log(a.q, a.tool, sent, True, text, 0, 0.0, "whole", "refused by the door; " + a.note)
        show(n, a.tool, True, text, 0, 0.0, "whole")
        return
    p, rpc, _ = server()
    line, wall = rpc({"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {"name": a.tool, "arguments": args}})
    close(p)
    if not line:
        text, is_error = "the server answered nothing and exited: " + p.stderr.read()[-2000:], True
    else:
        resp = json.loads(line)
        if "error" in resp:
            text, is_error = json.dumps(resp["error"]), True
        else:
            r = resp["result"]
            text = "".join(c.get("text", "") for c in r.get("content", []))
            is_error = bool(r.get("isError"))
    shown = "whole" if len(text.encode()) <= SHOWN else "head"
    n = log(a.q, a.tool, sent, is_error, text, len(line.encode()), wall, shown, a.note)
    show(n, a.tool, is_error, text, len(line.encode()), wall, shown)


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("open")
    c = sub.add_parser("call")
    c.add_argument("--q", required=True)
    c.add_argument("--note", default="")
    c.add_argument("tool")
    c.add_argument("args")
    a = ap.parse_args()
    if a.cmd == "open":
        open_cmd()
    else:
        call_cmd(a)


if __name__ == "__main__":
    main()
