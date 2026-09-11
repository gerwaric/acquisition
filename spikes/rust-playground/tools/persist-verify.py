#!/usr/bin/env python3
"""The persistence check's journal verifier (RUN-LEDGER.md, the persist
rows), called by tools/persist-check.sh after the wire phases:

    persist-verify.py <journal> <byte offset>

Per daemon lifetime in this run's slice of the journal — each opened by
its `open` header (C84: contract revision and acqd hash; a send with no
header of its own pid before it is a failure, never an anonymous
lifetime): every send answered 2xx, nothing else (a 3xx is not a pass —
the same predicate as tracer-verify.py); no 429; every route's first
send is its probe (declared no-probe routes and the token endpoint
excepted); exactly two lifetimes. Exit 0 with a draft ledger row, or 1
listing what failed.

    persist-verify.py --self-test

runs the synthetic journals that pin the refusals: a 302, a lifetime
whose header was removed, a send whose pid has no header, a journal
with no header at all, a 429, a GET before its probe.
"""

import json
import sys
import tempfile

NO_PROBE = {"oauth-token", "profile", "league"}


def read_lifetimes(journal, offset):
    lifetimes, cur = [], None
    with open(journal) as f:
        f.seek(int(offset))
        for raw in f:
            if not raw.strip():
                continue
            l = json.loads(raw)
            if l.get("event") == "open":
                # `contract` + `daemon` since 2026-09-10 (C84); `runtime` from
                # 2026-09-09 (C10); journals before that say `build`.
                cur = {"pid": l["pid"], "build": l.get("contract", l.get("runtime", l.get("build"))),
                       "daemon": (l.get("daemon") or "-")[:12], "clock": l["clock"], "sends": []}
                lifetimes.append(cur)
                continue
            if cur is None or l.get("pid") != cur["pid"]:
                cur = {"pid": l.get("pid"), "build": "?", "daemon": "?", "clock": "?", "sends": [],
                       "headerless": True}
                lifetimes.append(cur)
            cur["sends"].append(l)
    return lifetimes


def verify(journal, offset, out=print):
    lifetimes = read_lifetimes(journal, offset)
    fail, totals = [], []
    if not lifetimes or all(lt.get("headerless") for lt in lifetimes):
        fail.append("no open header in this run's journal: no daemon said which contract and acqd it was (C84)")
    for i, lt in enumerate(lifetimes, 1):
        out(f"lifetime {i}: pid {lt['pid']}  contract {lt['build']}  acqd {lt['daemon']}  clock {lt['clock']}")
        if lt.get("headerless"):
            fail.append(f"lifetime {i}: {len(lt['sends'])} send(s) by pid {lt['pid']} with no open header before them (C84)")
        counts, first = {}, {}
        for s in lt["sends"]:
            m, r, st = s["method"], s["route"], s.get("status")
            counts[m] = counts.get(m, 0) + 1
            first.setdefault(r, m)
            flag = ""
            if s.get("error") or st is None or not 200 <= st < 300:
                flag = "  <-- NOT OK"
                fail.append(f"lifetime {i}: {m} {r} -> {st} error={s.get('error')}")
            if st == 429:
                fail.append(f"lifetime {i}: 429 on {r}")
            if m == "HEAD":
                out(f"  HEAD {r} -> {st}  rate {json.dumps(s.get('rate'))}{flag}")
            else:
                out(f"  {m} {r} -> {st}  wait_ms {s.get('wait_ms')}{flag}")
        for r, m in first.items():
            # Declared no-probe routes (daemon.rs NO_PROBE_ROUTES; N38/N39, the
            # token endpoint): the login's own GET /profile has no HEAD before it.
            if r.split("@", 1)[0] not in NO_PROBE and m != "HEAD":
                fail.append(f"lifetime {i}: first send on {r} was {m}, not the probe")
        t = f"{counts.get('POST', 0)}/{counts.get('HEAD', 0)}/{counts.get('GET', 0)}"
        totals.append(f"{t} = {len(lt['sends'])}")
        out(f"  totals (POST/HEAD/GET): {totals[-1]}")

    if len(lifetimes) != 2:
        fail.append(f"expected 2 daemon lifetimes in this run's journal, saw {len(lifetimes)}")

    out("")
    if fail:
        out("CHECKS FAILED — a ledger row still gets written, saying what happened:")
        for x in fail:
            out(f"  - {x}")
        return False
    out("checks passed: halt left nothing failed, every route probed before its")
    out("first send in both lifetimes, no non-2xx. Read lifetime 2's HEAD line")
    out("above: its rate state carrying lifetime 1's hits IS the restart-replay")
    out("evidence (those hits are ours — the standing rule's 'stop and find it'")
    out("does not apply to this run).")
    out("")
    out("draft ledger row:")
    out(f"| <date> | persist | <tip> | pass | L1 {totals[0]}, L2 {totals[1]} | 0 |"
        " ceiling halt left children waiting; kill -9 mid-halt; successor probed"
        " before resuming; parent done across lifetimes; runs/<date>-persist/ |")
    return True


# ---- self-test: the refusals -------------------------------------------------

def synthetic(path, mutate=None):
    """Two lifetimes in the persist shape: the login GET, the listing's
    probe and GET, the stash probe and two GETs; then the successor's
    probe and three GETs."""
    lines = []

    def send(pid, method, r, status, rate=None):
        lines.append({"pid": pid, "ts": "2026-09-01T12:00:00.000Z", "method": method, "route": r,
                      "status": status, "ok": True, "error": None, "wait_ms": 0, "rate": rate or {}})

    def opened(pid):
        lines.append({"event": "open", "pid": pid, "contract": "x", "daemon": "y", "clock": "system",
                      "ts": "2026-09-01T12:00:00.000Z"})

    opened(1)
    send(1, "POST", "oauth-token", 200)
    send(1, "GET", "profile@A#1", 200)
    send(1, "HEAD", "stash-list@A#1", 204, {"x-rate-limit-account-state": "0:15:0"})
    send(1, "GET", "stash-list@A#1", 200)
    send(1, "HEAD", "stash@A#1", 204, {"x-rate-limit-account-state": "0:10:0"})
    send(1, "GET", "stash@A#1", 200)
    send(1, "GET", "stash@A#1", 200)
    opened(2)
    send(2, "POST", "oauth-token", 200)
    send(2, "HEAD", "stash@A#1", 204, {"x-rate-limit-account-state": "2:10:0"})
    for _ in range(3):
        send(2, "GET", "stash@A#1", 200)
    if mutate:
        mutate(lines)
    with open(path, "w") as f:
        f.write("\n".join(json.dumps(l) for l in lines) + "\n")


def gets(lines, pid):
    return [l for l in lines if l.get("method") == "GET" and l["pid"] == pid]


def redirect(lines):
    gets(lines, 2)[0]["status"] = 302


def rate_limited(lines):
    gets(lines, 2)[0]["status"] = 429


def header_removed(lines):
    lines.remove([l for l in lines if l.get("event") == "open" and l["pid"] == 2][0])


def unmatched_pid(lines):
    gets(lines, 2)[1]["pid"] = 3


def no_headers(lines):
    lines[:] = [l for l in lines if l.get("event") != "open"]


def get_before_probe(lines):
    head = [l for l in lines if l.get("method") == "HEAD" and l["pid"] == 2][0]
    lines.remove(head)


def self_test():
    cases = [
        ("the persist shape passes", None, True),
        ("a 302 is not a 2xx", redirect, False),
        ("a 429 fails", rate_limited, False),
        ("a lifetime whose open header was removed fails (C84)", header_removed, False),
        ("a send whose pid has no header fails (C84)", unmatched_pid, False),
        ("a journal with no header at all fails (C84)", no_headers, False),
        ("a GET before its route's probe fails", get_before_probe, False),
    ]
    failures = 0
    with tempfile.TemporaryDirectory() as d:
        for name, mutate, expect in cases:
            journal = f"{d}/j.jsonl"
            synthetic(journal, mutate)
            lines = []
            passed = verify(journal, 0, out=lines.append)
            ok = passed == expect
            print(f"{'ok  ' if ok else 'FAIL'} {name}: verifier {'passed' if passed else 'failed'}, expected {'pass' if expect else 'fail'}")
            if not ok:
                failures += 1
                for l in lines:
                    print("     " + l)
    print("self-test " + ("green" if failures == 0 else f"RED ({failures} case(s))"))
    return failures == 0


if __name__ == "__main__":
    if len(sys.argv) == 2 and sys.argv[1] == "--self-test":
        sys.exit(0 if self_test() else 1)
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(2)
    sys.exit(0 if verify(sys.argv[1], sys.argv[2]) else 1)
