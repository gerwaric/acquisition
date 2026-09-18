#!/usr/bin/env python3
"""Classify the trade site's pseudo stats by the search mechanism each needs.

Input: data/stats-2026-09-12.json (the site's stats capture). Output: one
count per class, on stdout. Written at the stage-5 harvest (2026-09-17) for
the owner's pseudo question; the classes are C94 (a weighted sum or count
over lines), C101 (a field, or a computed field) and C102 (out of reach:
needs the mod behind the line, S52). Classification is by id and text
pattern, so a new capture may need a new pattern; the counts it printed
are recorded in brainstorming-notes/28 §2b under C94 and C101.
"""
import collections
import json
import re
import sys
from pathlib import Path

path = Path(__file__).resolve().parent.parent / "data" / "stats-2026-09-12.json"
if len(sys.argv) > 1:
    path = Path(sys.argv[1])
doc = json.load(open(path))
groups = doc.get("result", doc)
entries = [e for g in groups if g.get("id") == "pseudo" for e in g["entries"]]


def classify(entry):
    i, t = entry["id"], entry["text"]
    if (
        i.startswith("pseudo.lake_")
        or "temple_" in i
        or "logbook_" in i
        or ("has_" in i and "influence" in i and "count" not in i)
    ):
        return "field or line presence (temple rooms, logbook, lake, influence)"
    if "jewellery_" in i or "map_" in i:
        return "field (catalyst quality, map properties)"
    if re.search(r"number_of_(prefix|suffix|affix|crafted_prefix|crafted_suffix|empty)", i) or "implicit_tier" in i:
        return "needs the mod behind the line (S52): out of reach"
    if "number_of_" in i or "count" in i or t.startswith("# total") or "notable" in i:
        return "count of lines (a total with weight 1, no slot)"
    if "percentile" in i:
        return "computed field (reference data + properties)"
    if t.startswith("Adds # to #"):
        return "range-valued total (low/high slot-wise)"
    return "weighted sum of lines (a C94 total)"


counts = collections.Counter(classify(e) for e in entries)
for name, n in counts.most_common():
    print(f"{n:4}  {name}")
print(f"{sum(counts.values()):4}  pseudo entries")
