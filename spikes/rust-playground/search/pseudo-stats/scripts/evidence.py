#!/usr/bin/env python3
"""What the owner's captures say of each (pseudo, line): ../data/evidence.csv.

Reads ../data/captures.json (scripts/captures.py) and the rows under test
(scripts/rows.py, each pseudo's latest version) and asks one question of every
fetched item, for every pseudo its search named: is what the site says of the
item what the rows give?

What the site says is the value it shows, or that it shows none; a pseudo
named inside a `not` group is one the site says the item has not. What the
rows give is the sum, over the item's lines, of each line's number times its
row's weight — for the ranged family the average of a line's two, shown in
both places — a line under an id or a twin the version does not count adding
nothing. **A sum of nothing is no value**: the site shows no total
where the lines cancel (c3), so the rows give none there either.

    evidence.py    # write ../data/evidence.csv, print what disagrees

A row is tested by the items that carry its line; a line the rows leave out
is tested at weight 0 the same way, for every pair ../data/candidates.csv
holds. A pair no captured item carries has no row. An item that disagrees is
printed with its lines and counted under `disagree` on each pair it carries:
the rows are wrong for it, and nothing here says which.

Numbers are exact: a displayed number is read as a decimal, never a float.
"""

import csv
import json
import re
import sys
from collections import defaultdict
from fractions import Fraction
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import rows as table  # noqa: E402

TRACK = Path(__file__).resolve().parents[1]
ROOT = TRACK.parent.parent
CAPTURES = TRACK / "data" / "captures.json"
CANDIDATES = TRACK / "data" / "candidates.csv"
OUT = TRACK / "data" / "evidence.csv"

# What limits a search's answer, said once and carried to every row it feeds.
CAVEATS = {}

NUMBER = r"[+-]?\d+(?:\.\d+)?"


def template(description):
    """A line's template: each number a `#`, the leading sign dropped."""
    return re.sub(NUMBER, "#", description.lstrip("+-")).replace("+#", "#")


def numbers(description):
    return [Fraction(n) for n in re.findall(NUMBER, description)]


def shown(text, description):
    """The numbers a pseudo's line shows, or None where the line is another's."""
    pattern = re.escape(text).replace(r"\#", f"({NUMBER})")
    found = re.fullmatch(pattern, description)
    return [Fraction(n) for n in found.groups()] if found else None


def asked(query):
    """(pseudo, the site says the item has it not) for every pseudo a query names."""
    return [
        (f["id"], g["type"] == "not")
        for g in query.get("stats", [])
        for f in g.get("filters", [])
        if f["id"].startswith("pseudo.")
    ]


def stat_of(entry):
    return (entry.get("hash") or "").removeprefix("stat.")


def given(version, slots, item):
    """What the rows give an item, as the site would show it, or None."""
    total = Fraction(0)
    counted = False
    for array, entries in item["lines"].items():
        if array == "pseudoMods":
            continue
        for e in entries:
            weight = table.weight_of(version, template(e["description"]), stat_of(e))
            read = numbers(e["description"])
            if weight is None or not read:
                continue
            if version["reads"] == "avg":
                if len(read) < 2:
                    continue
                total += (read[0] + read[1]) / 2 * weight
            else:
                total += read[0] * weight
            counted = True
    return [total] * slots if counted and total else None


def key(version, entry):
    """The pair a line tests: its template, with its twin or its id where that decides."""
    stat = stat_of(entry)
    t = template(entry["description"])
    if stat in version["never"]:
        return f"{t} [{stat}]"
    twins = {which for (row, which) in version["rows"] if row == t}
    return f"{t} (Local)" if stat in table.LOCAL and twins and twins != {"any"} else t


def main():
    if sys.argv[1:]:
        print(__doc__)
        return 2
    captures = json.loads(CAPTURES.read_text())["searches"]
    versions = table.latest()
    _, texts = table.stats()
    with CANDIDATES.open(newline="\n") as f:
        candidates = defaultdict(set)
        for r in csv.DictReader(f):
            candidates[r["pseudo"]].add(r["template"])

    pairs = defaultdict(lambda: {"agree": [], "disagree": [], "weight": None})
    readings = 0
    disagreeing = 0
    for name, capture in captures.items():
        for pseudo, site_says_not in asked(capture["returned_query"]):
            if pseudo not in versions:
                print(f"{name}: {pseudo} has no rows under test; skipped")
                continue
            version = versions[pseudo]
            slots = texts[pseudo].count("#")
            for index, item in enumerate(capture["fetched"]):
                readings += 1
                values = [
                    v
                    for e in item["lines"].get("pseudoMods", [])
                    if (v := shown(texts[pseudo], e["description"])) is not None
                ]
                if site_says_not and values:
                    raise SystemExit(f"{name}[{index}] shows a pseudo its search excluded")
                rows_give = given(version, slots, item)
                agrees = values == ([] if rows_give is None else [rows_give])
                lines = [
                    e
                    for array, entries in item["lines"].items()
                    if array != "pseudoMods"
                    for e in entries
                ]
                if not agrees:
                    disagreeing += 1
                    site = "none" if not values else " to ".join(table.plain(v) for v in values[0])
                    ours = "none" if rows_give is None else " to ".join(table.plain(v) for v in rows_give)
                    print(f"DISAGREES  {name}[{index}] {item['name']} {item['typeLine']}: "
                          f"{texts[pseudo]} — the site {site}, the rows {ours}")
                    for e in lines:
                        print(f"             {e['description']!r} {e.get('hash')}")
                named = {row for (row, _) in version["rows"]}
                for e in lines:
                    t = template(e["description"])
                    if t in named or t in candidates[pseudo]:
                        k = (pseudo, key(version, e))
                        where = f"{name}[{index}]"
                        bucket = pairs[k]["agree" if agrees else "disagree"]
                        if where not in bucket:
                            bucket.append(where)
                        weight = table.weight_of(version, t, stat_of(e))
                        pairs[k]["weight"] = Fraction(0) if weight is None else weight

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
                table.plain(seen["weight"]),
                len(seen["agree"]),
                len(seen["disagree"]),
                ";".join(seen["agree"] + seen["disagree"]),
                "; ".join(CAVEATS[s] for s in sorted(searches) if s in CAVEATS),
            ])
            print(f"{texts[pseudo]!r} · {t!r} at {table.plain(seen['weight'])}: "
                  f"{len(seen['agree'])} agree, {len(seen['disagree'])} disagree")
    print(f"{readings} readings of an item under a pseudo, {disagreeing} disagree; "
          f"{len(pairs)} pairs written to {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
