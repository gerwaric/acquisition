#!/usr/bin/env python3
"""The tracer rung's journal verifier (LIVE-TESTING.md, "Tracer rung"),
called by tools/tracer-rung.sh after the wire phases:

    tracer-verify.py <journal> <byte offset> <cycles.tsv> <login lifetimes 0|1>
                     <closed cycle or 0> <live|mock>

Checks, per daemon lifetime in this run's slice of the journal: every
route's first send is its probe (declared no-probe routes and the token
endpoint excepted); no non-2xx; every probe's reported hits, per window,
are at most this run's own GETs on that exact route inside that window
plus GGG's timing bucket for the window's position (N11/N12: 5 s on a
rule's first window, 60 s on the later ones — `bucket_for` in
ratelimit.rs), so the run's first probe on a route must report 0 and a
later probe may carry only the run's own earlier hits; each cycle's
lifetime holds exactly its plan's GETs, its probes, and one token POST.
Exit 0 with a draft ledger row, or 1 listing what failed.

    tracer-verify.py --self-test

runs the synthetic journals that pin the nonzero-hit branch, which mock
mode cannot reach (the mock's counters die with each daemon): own hits
inside a window pass, hits beyond ours in either window fail, and the
stash-list policy's second (60 s) window gets the 60 s bucket.
"""

import json
import sys
import tempfile
from datetime import datetime

# Routes taught by their first GET rather than a probe (declared route
# knowledge in daemon.rs), plus the token endpoint, which has no probe.
NO_PROBE = {"oauth-token", "profile", "league"}


def when(send):
    return datetime.fromisoformat(send["ts"].replace("Z", "+00:00"))


def reported_rules(rate):
    """Every (hits, period_seconds, window index) triple in every
    *-state header, index counted within its header — the position
    decides the timing bucket."""
    rules = []
    for key, value in (rate or {}).items():
        if key.endswith("-state"):
            for index, rule in enumerate(str(value).split(",")):
                parts = rule.split(":")
                if len(parts) >= 2 and parts[0].isdigit() and parts[1].isdigit():
                    rules.append((int(parts[0]), int(parts[1]), index))
    return rules


def bucket(index):
    """GGG's timing bucket past a window (N11/N12): 5 s on a rule's first
    window, 60 s on the sustained ones — a send that old may still be
    counted, so it is still 'ours'. Mirrors ratelimit.rs `bucket_for`."""
    return 5 if index == 0 else 60


def read_lifetimes(journal, offset):
    lifetimes, cur = [], None
    with open(journal) as f:
        f.seek(int(offset))
        for raw in f:
            if not raw.strip():
                continue
            line = json.loads(raw)
            if line.get("event") == "open":
                cur = {"pid": line["pid"], "build": line["build"], "clock": line["clock"], "sends": []}
                lifetimes.append(cur)
                continue
            if cur is None:
                cur = {"pid": line.get("pid"), "build": "?", "clock": "?", "sends": []}
                lifetimes.append(cur)
            cur["sends"].append(line)
    return lifetimes


def read_cycles(path):
    cycles = []
    with open(path) as f:
        for line in f:
            if line.strip():
                c, lt, logical, probes, ceiling, quote = line.rstrip("\n").split("\t")
                cycles.append({"cycle": int(c), "lifetime": int(lt), "logical": int(logical),
                               "probes": int(probes), "ceiling": int(ceiling), "quote": quote})
    return cycles


def verify(journal, offset, rows_path, login_lifetime, closed, mode, out=print):
    lifetimes = read_lifetimes(journal, offset)
    cycles = read_cycles(rows_path)
    fail, totals = [], []
    expected = login_lifetime + len(cycles)
    if mode == "mock" and len(lifetimes) == expected + 1 and not lifetimes[-1]["sends"]:
        # Mock mode ends with a throwaway daemon for the logout; it sends nothing.
        lifetimes.pop()
        out("(the mock-mode logout daemon, 0 sends, is left out of the count)")

    # This run's own counted GETs per exact route (account included), with
    # their times, so a probe's reported hits per window can be bounded by
    # what we ourselves sent inside that window (plus its bucket) — never by
    # the run's cumulative total, which would let outside traffic hide
    # behind aged-out sends of ours.
    ours = {}
    for i, lt in enumerate(lifetimes, 1):
        label = "login" if (login_lifetime and i == 1) else f"cycle {i - login_lifetime}"
        out(f"lifetime {i} ({label}): pid {lt['pid']}  build {lt['build']}  clock {lt['clock']}")
        counts, first = {}, {}
        for s in lt["sends"]:
            m, r, st = s["method"], s["route"], s.get("status")
            base = r.split("@", 1)[0]
            counts[m] = counts.get(m, 0) + 1
            first.setdefault(base, m)
            flag = ""
            if s.get("error") or st is None or st >= 400:
                flag = "  <-- NOT OK"
                fail.append(f"lifetime {i}: {m} {r} -> {st} error={s.get('error')}")
            if st == 429:
                fail.append(f"lifetime {i}: 429 on {r}")
            if m == "HEAD":
                t = when(s)
                rules = reported_rules(s.get("rate"))
                checks, over = [], False
                for hits, period, index in rules:
                    mine = sum(1 for sent in ours.get(r, [])
                               if (t - sent).total_seconds() <= period + bucket(index))
                    checks.append(f"{hits} of ours {mine} in {period}s(+{bucket(index)})")
                    if hits > mine:
                        over = True
                verdict = ""
                if over:
                    verdict = (f"  <-- reports more hits than this run sent inside the window"
                               f" ({'; '.join(checks)}): someone else is on this account")
                    fail.append(f"lifetime {i}: probe on {r} reported {'; '.join(checks)}")
                elif rules and not any(h for h, _, _ in rules):
                    verdict = "  (0 hits: nothing else on this account" + (
                        " — standing rule met)" if not ours.get(r)
                        else "; this run's earlier sends have aged out)")
                elif rules:
                    verdict = f"  (hits within this run's own sends in each window: {'; '.join(checks)} — expected)"
                out(f"  HEAD {r} -> {st}  rate {json.dumps(s.get('rate'))}{flag}{verdict}")
            else:
                out(f"  {m} {r} -> {st}  wait_ms {s.get('wait_ms')}{flag}")
                if m == "GET":
                    ours.setdefault(r, []).append(when(s))
        for base, m in first.items():
            if base not in NO_PROBE and m != "HEAD":
                fail.append(f"lifetime {i}: first send on {base} was {m}, not the probe")
        t = f"{counts.get('POST', 0)}/{counts.get('HEAD', 0)}/{counts.get('GET', 0)}"
        totals.append(f"{t} = {len(lt['sends'])}")
        out(f"  totals (POST/HEAD/GET): {totals[-1]}")

    if len(lifetimes) != expected:
        fail.append(f"expected {expected} daemon lifetime(s) in this run's journal, saw {len(lifetimes)}")
    if login_lifetime and lifetimes:
        login = lifetimes[0]["sends"]
        posts = sum(1 for s in login if s["method"] == "POST")
        gets = sum(1 for s in login if s["method"] == "GET" and s["route"].startswith("profile"))
        if posts != 1 or gets != 1 or len(login) != 2:
            fail.append(f"login lifetime: expected exactly one code-exchange POST and one GET /profile, saw {len(login)} sends")

    out("")
    out("plan vs journal, per cycle (the wire estimate's minimum should hold exactly):")
    for c in cycles:
        idx = c["lifetime"] - 1
        if idx >= len(lifetimes):
            fail.append(f"cycle {c['cycle']}: no journal lifetime for it")
            continue
        sends = lifetimes[idx]["sends"]
        gets = sum(1 for s in sends if s["method"] == "GET")
        heads = sum(1 for s in sends if s["method"] == "HEAD")
        posts = sum(1 for s in sends if s["method"] == "POST")
        ok = gets == c["logical"] and heads == c["probes"] and posts == 1 and len(sends) == c["ceiling"]
        out(f"  cycle {c['cycle']}: plan {c['logical']} request(s) + {c['probes']} probe(s) + 1 token POST"
            f" -> journal {posts} POST / {heads} HEAD / {gets} GET (ceiling {c['ceiling']}); {c['quote']}"
            + ("" if ok else "  <-- MISMATCH"))
        if not ok:
            fail.append(f"cycle {c['cycle']}: journal {posts}/{heads}/{gets} vs plan {c['logical']} + {c['probes']} probes + 1 POST")
    if closed:
        out(f"  cycle {closed}: empty plan, no-op apply, no daemon, nothing journaled — the loop closed")
    else:
        out("  the loop did not close within the cycle budget (recorded, not a failure)")

    out("")
    if fail:
        out("CHECKS FAILED — a ledger row still gets written, saying what happened:")
        for x in fail:
            out(f"  - {x}")
        return False
    quotes = ", ".join(f"c{c['cycle']} {c['quote'].split(':')[0]}" for c in cycles)
    out("checks passed: every route probed before its first send in every lifetime,")
    out("no probe reported hits beyond this run's own sends inside each window (the")
    out("first probe on each route saw 0), no non-2xx, and each cycle's journal matches")
    out("its plan exactly (no 429 re-sends: the estimate's minimum held). Quote outcomes: "
        + (quotes or "none") + ".")
    out("")
    out("draft ledger row:")
    lt = ", ".join(f"L{i + 1} {t}" for i, t in enumerate(totals))
    cyc = "; ".join(f"c{c['cycle']} {c['logical']} req" for c in cycles)
    out(f"| <date> | tracer | <tip> | pass | {lt} | 0 | policy → plan → apply → replan"
        f" ({cyc}{'; closed' if closed else '; not closed'}); each cycle's sends == its plan + probes + POST;"
        f" quote: {quotes or 'n/a'}; friction notes in the rung section; runs/<date>-tracer/ |")
    return True


# ---- self-test: the branches mock mode cannot reach --------------------------

def synthetic(path, route, probe2_state, gap_seconds, first_window="10", second_window="300"):
    """Two lifetimes: cycle 1 probes 0 hits and sends three GETs on `route`
    at T+1..3 s; cycle 2 opens `gap_seconds` later and its probe reports
    `probe2_state`, then two GETs. `first_window`/`second_window` name the
    rule periods in the state header."""
    lines = []

    def send(pid, ts, method, r, status, rate=None):
        lines.append(json.dumps({"pid": pid, "ts": ts, "method": method, "route": r, "status": status,
                                 "ok": True, "error": None, "wait_ms": 0, "rate": rate or {}}))

    def opened(pid, ts):
        lines.append(json.dumps({"event": "open", "pid": pid, "build": "x", "clock": "system", "ts": ts}))

    base = datetime.fromisoformat("2026-09-01T12:00:00+00:00")

    def at(seconds):
        return (base.replace(microsecond=0) + __import__("datetime").timedelta(seconds=seconds)).strftime("%Y-%m-%dT%H:%M:%S.000Z")

    opened(1, at(0))
    send(1, at(0.1), "POST", "oauth-token", 200)
    send(1, at(0.2), "HEAD", route, 204,
         {"x-rate-limit-account-state": f"0:{first_window}:0,0:{second_window}:0"})
    for i in range(3):
        send(1, at(1 + i), "GET", route, 200)
    opened(2, at(gap_seconds))
    send(2, at(gap_seconds + 0.1), "POST", "oauth-token", 200)
    send(2, at(gap_seconds + 0.2), "HEAD", route, 204, {"x-rate-limit-account-state": probe2_state})
    for i in range(2):
        send(2, at(gap_seconds + 1 + i), "GET", route, 200)
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def self_test():
    cases = [
        # (name, route, cycle-2 probe state, gap s, first window, second window, expect pass)
        ("own hits inside the 300 s window pass", "stash@A#1", "0:10:0,3:300:0", 100, "10", "300", True),
        ("a fourth hit in 300 s is not ours", "stash@A#1", "0:10:0,4:300:0", 100, "10", "300", False),
        ("a hit in the 10 s window is not ours", "stash@A#1", "1:10:0,3:300:0", 100, "10", "300", False),
        ("ours aged out of 300 s + 60 s bucket: any hit is foreign", "stash@A#1", "0:10:0,1:300:0", 400, "10", "300", False),
        ("stash-list: 3 hits 100 s later sit in the 60 s window's 60 s bucket", "stash-list@A#1", "0:15:0,3:60:0", 100, "15", "60", True),
        ("stash-list: 3 hits 130 s later are past 60 s + 60 s", "stash-list@A#1", "0:15:0,3:60:0", 130, "15", "60", False),
    ]
    failures = 0
    with tempfile.TemporaryDirectory() as d:
        rows = f"{d}/cycles.tsv"
        with open(rows, "w") as f:
            f.write("1\t1\t3\t1\t5\tquoted\n2\t2\t2\t1\t4\tquoted\n")
        for name, route, state, gap, w1, w2, expect in cases:
            journal = f"{d}/j.jsonl"
            synthetic(journal, route, state, gap, w1, w2)
            lines = []
            passed = verify(journal, 0, rows, 0, 3, "live", out=lines.append)
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
    if len(sys.argv) != 7:
        print(__doc__)
        sys.exit(2)
    journal, offset, rows, login, closed, mode = sys.argv[1:7]
    sys.exit(0 if verify(journal, offset, rows, int(login), int(closed), mode) else 1)
