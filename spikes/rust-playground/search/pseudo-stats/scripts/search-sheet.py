#!/usr/bin/env python3
"""Write the sheet of trade searches the owner runs: ../data/search-sheet.csv.

A search is a query object; the site's search id is that object, serialized
compactly, gzipped and base64url-encoded without padding (trade-query F9), so
a link is composed here and opened by a human in a browser. Nothing in this
script touches the network (C79; SURFACES.md, the trade site's rows: access
method `browser`).

    search-sheet.py              # write ../data/search-sheet.csv, print the links
    search-sheet.py --self-test  # compose each captured request and compare ids

The self-test reads `../../trade-query/raw/searches/` (local only, never
committed): for every `<q>-request.json` with a `<q>-search.json` beside it,
the id composed from the request must equal the id the site returned. It
skips, saying so, where the captures are absent.

Each search names what it decides. A search whose answer would read the same
under two hypotheses is not a search: every one carries a control — a pseudo
the returned items are known to have — so that an absent value is the site's
answer and never the method's failure.
"""

import base64
import csv
import json
import struct
import sys
import zlib
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
OUT = TRACK / "data" / "search-sheet.csv"
CAPTURED = TRACK.parent / "trade-query" / "raw" / "searches"

SITE = "https://www.pathofexile.com/trade/search"
LEAGUE = "Standard"

# The ten bytes every captured id opens with: gzip, deflate, no flags, no
# time, no extra flags, OS 3.
GZIP_HEADER = bytes.fromhex("1f8b0800000000000003")


def compose(query):
    """The site's id for a query object."""
    body = json.dumps(query, separators=(",", ":"), ensure_ascii=False).encode()
    deflate = zlib.compressobj(6, zlib.DEFLATED, -15)
    packed = deflate.compress(body) + deflate.flush()
    trailer = struct.pack("<II", zlib.crc32(body), len(body))
    whole = GZIP_HEADER + packed + trailer
    return base64.urlsafe_b64encode(whole).decode().rstrip("=")


def link(query, league=LEAGUE):
    return f"{SITE}/{league}/{compose(query)}"


def query(stats, rarity="rare"):
    """A query in the shape the owner's captures pin (trade-query F1)."""
    return {
        "status": {"option": "any"},
        "stats": stats,
        "filters": {
            "type_filters": {
                "disabled": False,
                "filters": {"rarity": {"option": rarity}},
            },
            "trade_filters": {
                "disabled": True,
                "filters": {"sale_type": {"option": "any"}},
            },
        },
    }


def group(kind, ids, value=None):
    out = {"type": kind, "filters": [{"id": i, "disabled": False} for i in ids]}
    if value is not None:
        out["value"] = value
    return out


# The pilot: two searches that settle the method before the batch is written.
# Both are new to the site, so either opening shows a composed link is taken.
PILOT = [
    {
        "search": "p1",
        "decides": (
            "the method: a composed link opens. The evidence: whether "
            "`+# to Strength and Intelligence` counts toward "
            "`+# total maximum Life`, and at what weight — the total less "
            "the item's life lines and half its Strength lines is the answer"
        ),
        "control": "pseudo.pseudo_total_life is required, so every item shows it",
        "query": query(
            [
                group("and", ["pseudo.pseudo_total_life"]),
                # Two ids display this one text (trade-query F4), so the line
                # goes out as a count of at least one (the contract detail, C99).
                group(
                    "count",
                    ["explicit.stat_1535626285", "explicit.stat_2543977012"],
                    {"min": 1},
                ),
            ]
        ),
    },
    {
        "search": "p2",
        "decides": (
            "the method: an `if` group shows a pseudo's value without "
            "filtering by it. The evidence: whether "
            "`#% increased Attack and Cast Speed` counts toward "
            "`+#% total Attack Speed` and `+#% total Cast Speed`"
        ),
        "control": (
            "every item carries `+#% to Fire Resistance`, so "
            "pseudo.pseudo_total_fire_resistance shown and a speed total "
            "absent is the site's answer; none of the three shown means an "
            "`if` group shows nothing"
        ),
        "query": query(
            [
                group(
                    "and",
                    ["explicit.stat_2672805335", "explicit.stat_3372524247"],
                ),
                group(
                    "if",
                    [
                        "pseudo.pseudo_total_fire_resistance",
                        "pseudo.pseudo_total_attack_speed",
                        "pseudo.pseudo_total_cast_speed",
                    ],
                ),
            ]
        ),
    },
]

SEARCHES = PILOT


def self_test():
    requests = sorted(CAPTURED.glob("*-request.json"))
    if not requests:
        print(f"skipped: no captures under {CAPTURED} (raw/ is local only)")
        return 0
    failed = 0
    checked = 0
    for request in requests:
        answer = request.with_name(request.name.replace("-request", "-search"))
        if not answer.exists() or answer.stat().st_size == 0:
            continue
        want = json.loads(answer.read_text())["id"]
        got = compose(json.loads(request.read_text())["query"])
        checked += 1
        if got != want:
            failed += 1
            print(f"DIFFER  {request.name}")
    print(f"{checked} captured ids composed, {failed} differ")
    return 1 if failed else 0


def main():
    if sys.argv[1:] == ["--self-test"]:
        return self_test()
    if sys.argv[1:]:
        print(__doc__)
        return 2
    seen = set()
    with OUT.open("w", newline="") as out:
        sheet = csv.writer(out, lineterminator="\n")
        sheet.writerow(["search", "decides", "control", "league", "link", "query"])
        for row in SEARCHES:
            if row["search"] in seen:
                raise SystemExit(f"two searches named {row['search']}")
            seen.add(row["search"])
            body = json.dumps(row["query"], separators=(",", ":"), ensure_ascii=False)
            sheet.writerow(
                [
                    row["search"],
                    row["decides"],
                    row["control"],
                    LEAGUE,
                    link(row["query"]),
                    body,
                ]
            )
            print(f"{row['search']}  {link(row['query'])}")
    print(f"{len(SEARCHES)} searches written to {OUT.relative_to(TRACK.parent.parent)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
