#!/usr/bin/env python3
"""The totals table's changes as rows: search/pseudo-stats/data/table-changes.csv.

What the plan's step 9c closes on: every change the captures ask of
the totals table as shipped at v1 (search/pseudo-stats/data/totals-v1.toml), one row each, with the
evidence that asks for it — none applied here, and none to be applied without
its row. Reads tools/trade_rows.py (each pseudo's versions, every version's `why`
naming its captures), search/pseudo-stats/data/captures.json and search/pseudo-stats/data/search-sheet.csv.

    tools/trade-changes.py    # write search/pseudo-stats/data/table-changes.csv, print the counts

A row's `change` is one of: `new total` (a pseudo no table ships), `add row`,
`remove row`, `twin` (a row that means one of two stats displaying its text,
which the build answers by what the item is), `not mimicked` (an id or a row
the site leaves out and the search counts, by the owner's ruling, or waiting
for it where the evidence says so), `rule` (what holds for every total),
`no sum` (a pseudo that is a reading of other totals, which no row can say:
the reading is in `template`, and it is written as a reading, never as a
total), `limit` (what
a total read from an item's text cannot match of the site's: `weight` is how
far under the site's a line may be, and `twin_or_reads` whether the owner has
ruled on it — a total whose limit is not ruled is not written).

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
import trade_rows as table  # noqa: E402

TRACK = Path(__file__).resolve().parents[1] / "search" / "pseudo-stats"
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
    ("a row's `reduced` spelling is the row, counted below nothing",
     "g007, g034, h042, h044, h046, h048, h050, h052: the line is under the `increased` spelling's "
     "id, and the site shows the total below nothing on 60 readings of the 61 that carry one, the "
     "other a sum of nothing (g034); left out, 59 readings of round six disagree. The owner, "
     "2026-09-27: \"yes, we need to be able to find reduced lines and totals. There are "
     "occasionally niche builds for which this is critically important.\""),
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
# The same id under total mana, ruled of it apart: what the owner ruled at 9c
# he ruled of total life, Strength and Intelligence.
NOT_MIMICKED_MANA = (
    "the site leaves this id out, as it does of total life, Strength and Intelligence; the "
    "search counts the line wherever it is displayed. The owner, 2026-09-27: \"yes, count it "
    "for mana.\""
)
# A limit the owner has ruled on: {total: his words, dated}. A total whose
# limit is not here waits, and the table is written without it.
LIMITS_RULED = {
    "life_regen": "2026-09-27: \"yes, let's go with what we can observe directly from the text "
                  "we have.\"",
}


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
        if name in table.DERIVED:
            kind, of, why = table.DERIVED[name]
            status = "open" if latest["pseudo"] in wrong else "explained"
            counts[status] += 1
            out.append([name, latest["pseudo"], texts[latest["pseudo"]], "no sum",
                        f"the {kind} of {', '.join(of)}", "", "", why, status])
            continue
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
            out.append(head + ["not mimicked", stat, "", "",
                               NOT_MIMICKED_MANA if name == "total_mana" else NOT_MIMICKED, status])
        if latest["cut"]:
            why = next(v["why"] for v in history if v["cut"])
            ruled = LIMITS_RULED.get(name)
            out.append(head + ["limit", "", table.plain(latest["cut"]),
                               "ruled" if ruled else "not ruled",
                               why + (f". The owner, {ruled}" if ruled else ""), status])
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
