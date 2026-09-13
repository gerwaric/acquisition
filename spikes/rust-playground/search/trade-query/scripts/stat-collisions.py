#!/usr/bin/env python3
"""Where the trade stats vocabulary is not a function of display text.

Reads data/stats-<date>.json and writes data/stat-collisions.csv: one row per
(category, text) whose text maps to more than one stat id within that
category — the cases where a display line alone cannot pick the trade
stat — plus data/stat-summary.csv: per category, entries, distinct texts,
texts with n placeholders, texts with a newline, entries with options,
texts carrying "(Local)".
"""
import csv, json, sys, collections
from pathlib import Path
root = Path(__file__).resolve().parent.parent
date = sys.argv[1] if len(sys.argv) > 1 else "2026-09-12"
cats = json.load(open(root / "data" / f"stats-{date}.json"))["result"]

with open(root / "data" / "stat-collisions.csv", "w", newline="") as f:  # LF: csv defaults to CRLF
    w = csv.writer(f, lineterminator="\n"); w.writerow(["category", "text", "ids", "id_kinds"])
    n = 0
    for c in cats:
        by = collections.defaultdict(list)
        for e in c["entries"]:
            by[e["text"]].append(e["id"].split(".", 1)[1])
        for t, ids in sorted(by.items()):
            if len(ids) > 1:
                kinds = sorted({i.split("_")[0] if "_" in i else i for i in ids})
                w.writerow([c["id"], t, " ".join(ids), " ".join(kinds)]); n += 1
print("collisions:", n)

with open(root / "data" / "stat-summary.csv", "w", newline="") as f:  # LF: csv defaults to CRLF
    w = csv.writer(f, lineterminator="\n")
    w.writerow(["category", "label", "entries", "distinct_texts", "hash0", "hash1", "hash2", "hash3plus", "multiline", "with_options", "local_suffix", "id_kinds"])
    for c in cats:
        texts = [e["text"] for e in c["entries"]]
        h = collections.Counter(min(t.count("#"), 3) for t in texts)
        kinds = collections.Counter(e["id"].split(".", 1)[1].split("_")[0] for e in c["entries"])
        w.writerow([c["id"], c["label"], len(texts), len(set(texts)), h[0], h[1], h[2], h[3],
                    sum("\n" in t for t in texts), sum("option" in e for e in c["entries"]),
                    sum("(Local)" in t for t in texts), " ".join(f"{k}:{v}" for k, v in sorted(kinds.items()))])
# cross-category: the numeric stat id shared across categories
idcats = collections.defaultdict(set)
for c in cats:
    for e in c["entries"]:
        idcats[e["id"].split(".", 1)[1]].add(c["id"])
shared = collections.Counter(len(v) for v in idcats.values())
print("distinct stat keys:", len(idcats), "by number of categories they appear in:", dict(sorted(shared.items())))
