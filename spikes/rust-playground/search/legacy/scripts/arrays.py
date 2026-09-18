#!/usr/bin/env python3
"""Every mod array the owner's corpus carries besides implicit and explicit, line template by line
template, over every rarity — not the uniques alone.

Input:  the two store backups in raw/.
Output: data/other-arrays.csv — one row per (array, line template).

No line is judged here. Whether a template can still be made is the README's sources-read table.
"""

import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import common  # noqa: E402

FIT_ARRAYS = ("implicitMods", "explicitMods")


def main():
    items = common.read_items()
    rows = collections.defaultdict(lambda: {"items": 0, "lines": 0, "league": collections.Counter(),
                                            "rarity": collections.Counter(), "cls": collections.Counter(),
                                            "example": None})
    per_array_items = collections.Counter()
    carriers = set()
    flagged = collections.defaultdict(lambda: collections.Counter())
    for it in items:
        j = it.json
        rarity = j.get("frameTypeId", "?")
        cls = common.icon_dir(j.get("icon", ""))
        seen_arrays = set()
        for array, texts, flags in common.mod_arrays(j):
            if array in FIT_ARRAYS:
                for f in flags:
                    flagged[array][f] += 1
                continue
            if not texts:
                continue
            seen_arrays.add(array)
            for f in flags:
                flagged[array][f] += 1
            seen = set()
            for text in texts:
                tpl = common.tokenize(text)[0]
                row = rows[(array, tpl)]
                row["lines"] += 1
                if row["example"] is None:
                    row["example"] = text[:110]
                if tpl not in seen:
                    seen.add(tpl)
                    row["items"] += 1
                    row["league"][it.league] += 1
                    row["rarity"][rarity] += 1
                    row["cls"][cls] += 1
        for array in seen_arrays:
            per_array_items[array] += 1
        if seen_arrays:
            carriers.add(it.id)

    out = []
    for (array, tpl), row in sorted(rows.items(), key=lambda kv: (kv[0][0], -kv[1]["items"], kv[0][1])):
        out.append([array, tpl, row["items"], row["lines"],
                    ";".join(f"{k}:{v}" for k, v in sorted(row["league"].items())),
                    ";".join(f"{k}:{v}" for k, v in row["rarity"].most_common()),
                    ";".join(f"{k}:{v}" for k, v in row["cls"].most_common(3)),
                    row["example"]])
    common.write_csv("other-arrays.csv",
                     ["array", "template", "items", "lines", "by_league", "by_rarity",
                      "by_icon_dir", "example"], out)

    n = len(items)
    print(f"  items in both stores (deduplicated by GGG id): {n:,}")
    print(f"  items carrying at least one line outside implicit and explicit: {len(carriers):,} "
          f"({len(carriers) / n:.3f} of the corpus)")
    print("  per array — items / distinct line templates / lines:")
    tpl_per_array = collections.Counter()
    lines_per_array = collections.Counter()
    for (array, _), row in rows.items():
        tpl_per_array[array] += 1
        lines_per_array[array] += row["lines"]
    for array, c in per_array_items.most_common():
        print(f"    {array:<16} {c:6,d} / {tpl_per_array[array]:5,d} / {lines_per_array[array]:6,d}")
    print("  per array, items by rarity:")
    for array in sorted(per_array_items, key=lambda a: -per_array_items[a]):
        r = collections.Counter()
        for (a, _), row in rows.items():
            if a == array:
                r.update(row["rarity"])
        print(f"    {array:<16} " + ", ".join(f"{k} {v:,}" for k, v in r.most_common()))
    print("  per array, items by league:")
    for array in sorted(per_array_items, key=lambda a: -per_array_items[a]):
        lg = collections.Counter()
        for (a, _), row in rows.items():
            if a == array:
                lg.update(row["league"])
        print(f"    {array:<16} " + ", ".join(f"{k} {v:,}" for k, v in lg.most_common()))
    print("  per array, the commonest icon directory (the only item-class evidence the JSON gives):")
    for array in sorted(per_array_items, key=lambda a: -per_array_items[a]):
        cl = collections.Counter()
        for (a, _), row in rows.items():
            if a == array:
                cl.update(row["cls"])
        print(f"    {array:<16} " + ", ".join(f"{k} {v:,}" for k, v in cl.most_common(8)))
    print("  flags carried by lines, by array (a flag, not an array of its own):")
    for array, c in sorted(flagged.items()):
        print(f"    {array:<16} " + ", ".join(f"{k} {v:,}" for k, v in c.most_common()))
    print("  the five commonest templates per array:")
    for array in sorted(per_array_items, key=lambda a: -per_array_items[a]):
        print(f"    {array}:")
        top = sorted(((tpl, row) for (a, tpl), row in rows.items() if a == array),
                     key=lambda kv: -kv[1]["items"])[:5]
        for tpl, row in top:
            print(f"      {row['items']:6,d}  {tpl[:88]}")


if __name__ == "__main__":
    sys.exit(main())
