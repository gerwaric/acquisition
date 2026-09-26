#!/usr/bin/env python3
"""What the owner's captures say of each (pseudo, line): ../data/evidence.csv.

Reads ../data/captures.json (scripts/captures.py), the totals the build ships
(crates/acquisition-search/reference/totals-v1.toml) and PROPOSED below, and
asks one question of every fetched item, for every pseudo its search named:
is the value the site shows the sum, over the item's lines, of each line's
number times the weight its row gives — and no value where that sum is
nothing? A row is tested by the items that carry its line; a line the rows
leave out is tested at weight 0 the same way, for every pair
../data/candidates.csv holds.

    evidence.py    # write ../data/evidence.csv, print what disagrees

A pair no captured item carries has no row. An item that disagrees is printed
with its lines, and its pairs count it under `disagree`: the rows are wrong
for it, and nothing here says which.

Numbers are exact: a displayed number is read as a decimal, never a float.
"""

import csv
import json
import re
import sys
import tomllib
from collections import defaultdict
from fractions import Fraction
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
ROOT = TRACK.parent.parent
CAPTURES = TRACK / "data" / "captures.json"
CANDIDATES = TRACK / "data" / "candidates.csv"
STATS = TRACK.parent / "trade-query" / "data" / "stats-2026-09-12.json"
TOTALS = ROOT / "crates" / "acquisition-search" / "reference" / "totals-v1.toml"
OUT = TRACK / "data" / "evidence.csv"

# Rows under test that no table ships yet, by the site's id: template, weight.
# Each is a hypothesis; what the captures say of it is this script's output.
PROPOSED = {
    "pseudo.pseudo_total_life": [
        ("# to maximum Life", "1"),
        ("# to Strength", "0.5"),
        ("# to Strength and Dexterity", "0.5"),
        ("# to Strength and Intelligence", "0.5"),
        ("# to all Attributes", "0.5"),
    ],
}

# What limits a search's answer, said once and carried to every row it feeds.
CAVEATS = {
    "p2": (
        "the control was the first member of its `if` group: that every "
        "member displays is not yet shown"
    ),
}

NUMBER = r"[+-]?\d+(?:\.\d+)?"


def template(description):
    """A line's template: each number a `#`, the leading sign dropped."""
    return re.sub(NUMBER, "#", description.lstrip("+-")).replace("+#", "#")


def numbers(description):
    return [Fraction(n) for n in re.findall(NUMBER, description)]


def shown(text, description):
    """The value a pseudo's line shows, or None where the line is another's."""
    pattern = re.escape(text).replace(r"\#", f"({NUMBER})")
    found = re.fullmatch(pattern, description)
    return Fraction(found.group(1)) if found else None


def rows_by_pseudo():
    rows = {}
    for total in tomllib.loads(TOTALS.read_text())["total"]:
        if total.get("ranged"):
            continue
        rows["pseudo." + total["site"]] = {
            r["template"]: Fraction(str(r["weight"])) for r in total["rows"] if r.get("slot", "arg1") == "arg1"
        }
    for pseudo, proposed in PROPOSED.items():
        rows[pseudo] = {t: Fraction(w) for t, w in proposed}
    return rows


def pseudo_texts():
    stats = json.loads(STATS.read_text())
    group = next(g for g in stats["result"] if g["id"] == "pseudo")
    return {e["id"]: e["text"] for e in group["entries"]}


def asked(query):
    return [
        f["id"]
        for g in query.get("stats", [])
        for f in g.get("filters", [])
        if f["id"].startswith("pseudo.")
    ]


def plain(value):
    return str(value.numerator) if value.denominator == 1 else str(float(value))


def main():
    if sys.argv[1:]:
        print(__doc__)
        return 2
    captures = json.loads(CAPTURES.read_text())["searches"]
    rows = rows_by_pseudo()
    texts = pseudo_texts()
    with CANDIDATES.open(newline="\n") as f:
        candidates = defaultdict(set)
        for r in csv.DictReader(f):
            candidates[r["pseudo"]].add(r["template"])

    pairs = defaultdict(lambda: {"agree": [], "disagree": []})
    items_read = 0
    disagreeing = 0
    for name, capture in captures.items():
        for pseudo in asked(capture["returned_query"]):
            if pseudo not in rows:
                print(f"{name}: {pseudo} has no rows under test; skipped")
                continue
            for index, item in enumerate(capture["fetched"]):
                items_read += 1
                lines = [
                    e["description"]
                    for array, entries in item["lines"].items()
                    if array != "pseudoMods"
                    for e in entries
                ]
                values = [
                    v
                    for e in item["lines"].get("pseudoMods", [])
                    if (v := shown(texts[pseudo], e["description"])) is not None
                ]
                carried = defaultdict(Fraction)
                for description in lines:
                    t = template(description)
                    if t in rows[pseudo] and numbers(description):
                        carried[t] += numbers(description)[0] * rows[pseudo][t]
                computed = sum(carried.values(), Fraction(0))
                agrees = (values == [computed]) if carried else (values == [])
                if not agrees:
                    disagreeing += 1
                    print(f"DISAGREES  {name}[{index}] {item['typeLine']}: {texts[pseudo]} "
                          f"shown {[plain(v) for v in values]}, the rows give {plain(computed)}")
                    for description in lines:
                        print(f"             {description!r}")
                here = {template(d) for d in lines}
                for t in here & (set(rows[pseudo]) | candidates[pseudo]):
                    pairs[(pseudo, t)]["agree" if agrees else "disagree"].append(f"{name}[{index}]")

    with OUT.open("w", newline="") as out:
        sheet = csv.writer(out, lineterminator="\n")
        sheet.writerow(["pseudo", "pseudo_text", "template", "weight_tested", "agree",
                        "disagree", "items", "caveat"])
        for (pseudo, t), seen in sorted(pairs.items()):
            searches = {i.split("[")[0] for i in seen["agree"] + seen["disagree"]}
            sheet.writerow([
                pseudo,
                texts[pseudo],
                t,
                plain(rows[pseudo].get(t, Fraction(0))),
                len(seen["agree"]),
                len(seen["disagree"]),
                ";".join(seen["agree"] + seen["disagree"]),
                "; ".join(CAVEATS[s] for s in sorted(searches) if s in CAVEATS),
            ])
            print(f"{texts[pseudo]!r} · {t!r} at {plain(rows[pseudo].get(t, Fraction(0)))}: "
                  f"{len(seen['agree'])} agree, {len(seen['disagree'])} disagree")
    print(f"{items_read} readings of an item under a pseudo, {disagreeing} disagree; "
          f"{len(pairs)} pairs written to {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
