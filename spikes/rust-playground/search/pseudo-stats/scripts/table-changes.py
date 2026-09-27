#!/usr/bin/env python3
"""The totals table's changes as rows: ../data/table-changes.csv.

What the plan's step 9c closes on: every change the captures ask of
the totals table as shipped at v1 (../data/totals-v1.toml), one row each, with the
evidence that asks for it — none applied here, and none to be applied without
its row. Reads scripts/rows.py (each pseudo's versions, every version's `why`
naming its captures), ../data/captures.json and ../data/search-sheet.csv.

    table-changes.py    # write ../data/table-changes.csv, print the counts

A row's `change` is one of: `new total` (a pseudo no table ships), `add row`,
`remove row`, `twin` (a row that means one of two stats displaying its text,
which the build answers by what the item is), `not mimicked` (an id or a row
the site leaves out and the search counts, by the owner's ruling), `rule`
(what holds for every total).

A total's `status` is what the site said of its latest rows: `closed` — a
complete and a sound check each found nothing; `explained` — a check found
items, and every item fetched shows what the rows give; `open` otherwise.
"""

import csv
import json
import re
import sys
import tomllib
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import rows as table  # noqa: E402

TRACK = Path(__file__).resolve().parents[1]
ROOT = TRACK.parent.parent
CAPTURES = TRACK / "data" / "captures.json"
SHEET = TRACK / "data" / "search-sheet.csv"
EVIDENCE = TRACK / "data" / "evidence.csv"
OUT = TRACK / "data" / "table-changes.csv"

# The owner's rulings, 2026-09-26, verbatim; and what the captures show of
# every total alike.
RULES = [
    ("a total whose lines sum to nothing is absent, never 0",
     "c3, d01, e043: the site shows no total where the lines cancel. The owner: "
     "\"Yes, let's make it absent to match the site.\""),
    ("a ranged total is the average of a line's two numbers",
     "round two: every reading of a ranged pseudo; the low number read instead disagrees on all"),
]
# Rows the site leaves out and the table keeps, by the owner's ruling.
KEPT = {
    ("total_skill_gem_levels", "# to Level of Socketed Skill Gems"): (
        "e103, f030, f031: the site leaves the pseudo's own text out of it. The owner, "
        "2026-09-26: \"yes, include the socketed skill gems\""
    ),
}
ELDRITCH = " The owner, 2026-09-26: \"include the eldritch mods\""
NOT_MIMICKED = (
    "the site leaves this id out; the search counts the line wherever it is displayed. "
    "The owner: \"I believe this is a bug. Let's count the mod\""
)


def checks():
    """{name: {kind: (version, [(search, total), …])}}, the latest version asked of each."""
    captures = json.loads(CAPTURES.read_text())["searches"]
    last = defaultdict(dict)
    with SHEET.open(newline="\n") as f:
        for r in csv.DictReader(f):
            found = re.match(r"(\w+) v(\d+), (complete|sound|what carries)", r["decides"])
            if not found or r["search"] not in captures:
                continue
            name, number = found.group(1), int(found.group(2))
            kind = "sound" if found.group(3) == "sound" else "complete"
            total = captures[r["search"]]["search"]["total"]
            held = last[name].get(kind)
            if held is None or number > held[0]:
                last[name][kind] = (number, [(r["search"], total)])
            elif number == held[0]:
                held[1].append((r["search"], total))
    # Total life was asked before the rounds were named by version.
    last["total_life"] = {"complete": (1, [("c1", 0)]), "sound": (2, [("d01", 1)])}
    return last


def disagreeing():
    with EVIDENCE.open(newline="\n") as f:
        return {r["pseudo"] for r in csv.DictReader(f) if int(r["disagree"])}


def main():
    if sys.argv[1:]:
        print(__doc__)
        return 2
    shipped = {t["name"]: {(r["template"], "any"): str(r["weight"]) for r in t["rows"]}
               for t in tomllib.loads((TRACK / "data" / "totals-v1.toml")
                                      .read_text())["total"]}
    _, texts = table.stats()
    asked = checks()
    wrong = disagreeing()
    out = []
    counts = defaultdict(int)
    for rule, why in RULES:
        out.append(["every total", "", "", "rule", rule, "", "", why, ""])
    for name, history in table.versions().items():
        latest = history[-1]
        kinds = asked.get(name, {})
        found = {k: sum(t for _, t in v[1]) for k, v in kinds.items()}
        if latest["pseudo"] in wrong:
            status = "open"
        elif found.get("complete") == 0 and found.get("sound") == 0:
            status = "closed"
        elif name == "total_life":
            status = "closed"  # d01's one item is a sum of nothing
        else:
            status = "explained"
        counts[status] += 1
        why_of = {}
        for version in history:
            for key in version["rows"]:
                why_of.setdefault(key, version["why"])
        head = [name, latest["pseudo"], texts[latest["pseudo"]]]
        before = shipped.get(name)
        if before is None:
            out.append(head + ["new total", "", "", latest["reads"],
                               f"no table ships it; {len(latest['rows'])} rows below", status])
            before = {}
        for (t, which), w in latest["rows"].items():
            if (t, which) in before and before[(t, which)] == table.plain(w):
                continue
            change = "twin" if which != "any" else "add row"
            why = why_of[(t, which)] + (ELDRITCH if t.startswith("While a ") else "")
            out.append(head + [change, t, table.plain(w), which, why, status])
        for (t, which) in before:
            if (t, which) not in latest["rows"]:
                if (name, t) in KEPT:
                    out.append(head + ["not mimicked", t, before[(t, which)], which,
                                       KEPT[(name, t)], status])
                    continue
                gone = next(v["why"] for v in history if (t, which) not in v["rows"])
                out.append(head + ["remove row", t, before[(t, which)], which, gone, status])
        for stat in latest["never"]:
            out.append(head + ["not mimicked", stat, "", "", NOT_MIMICKED, status])
    with OUT.open("w", newline="") as f:
        sheet = csv.writer(f, lineterminator="\n")
        sheet.writerow(["total", "site_id", "site_text", "change", "template", "weight",
                        "twin_or_reads", "evidence", "status"])
        sheet.writerows(out)
    by = defaultdict(int)
    for r in out:
        by[r[3]] += 1
    print(f"{len(out)} rows written to {OUT.relative_to(ROOT)}: "
          + ", ".join(f"{n} {k}" for k, n in sorted(by.items())))
    print("totals by status: " + ", ".join(f"{n} {k}" for k, n in sorted(counts.items())))
    unchanged = [n for n in table.versions() if n not in {r[0] for r in out}]
    print(f"{len(unchanged)} shipped totals need no change")
    return 0


if __name__ == "__main__":
    sys.exit(main())
