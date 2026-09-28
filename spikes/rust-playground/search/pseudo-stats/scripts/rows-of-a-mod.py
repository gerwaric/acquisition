#!/usr/bin/env python3
"""Step 9c4 of search/BUILD-PLAN.md: the mods displayed over several rows,
on the owner's store copy and among the trade site's stat texts — what a
row of one names, and what it cannot.

    cargo build --workspace && python3 search/item-facts/scripts/m2-differential.py   # writes raw/m2/rust.json
    python3 search/pseudo-stats/scripts/rows-of-a-mod.py

Input: the deriver's own census of the copy
(search/item-facts/raw/m2/rust.json — a row per (source, template) with
its items and lines, in the crate's template form, a row break `\\n`), the
trade site's stats (search/trade-query/data/stats-2026-09-12.json) and the
shipped totals table. Lines, not items: the census counts items per
(source, template), so an item is counted once per incidence. Never in
the gate: the input is raw/.
"""
import collections
import glob
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CENSUS = os.path.join(ROOT, "search", "item-facts", "raw", "m2", "rust.json")
STATS = os.path.join(ROOT, "search", "trade-query", "data", "stats-2026-09-12.json")
TABLES = os.path.join(ROOT, "crates", "acquisition-search", "reference", "totals-v*.toml")


def named_by_a_total():
    """The templates the shipped table's rows name, folded, each with its totals."""
    (table,) = glob.glob(TABLES)
    with open(table, encoding="utf-8") as f:
        text = f.read()
    by_template = collections.defaultdict(list)
    for block in text.split("[[total]]")[1:]:
        name = re.search(r'^name = "(.*)"$', block, re.M).group(1)
        for template in re.findall(r'\{ template = "((?:[^"\\]|\\.)*)"', block):
            template = template.replace('\\"', '"').replace("\\\\", "\\")
            by_template[template.lower()].append(name)
    return os.path.basename(table), by_template


def rows_of(template):
    return [row for row in template.split("\n") if row]


def main():
    if not os.path.exists(CENSUS):
        sys.exit(f"{CENSUS} is missing: run search/item-facts/scripts/m2-differential.py after cargo build")
    with open(CENSUS, encoding="utf-8") as f:
        census = json.load(f)
    table, totals = named_by_a_total()

    whole = collections.Counter()
    several = collections.Counter()
    for row in census["rows"]:
        whole[row["template"]] += row["lines"]
        if "\n" in row["template"]:
            several[row["template"]] += row["lines"]
    lines = sum(whole.values())
    print(f"the copy: {census['items']:,} items, {lines:,} lines, {len(whole):,} templates")
    print(f"mods of several rows: {sum(several.values()):,} lines of {len(several):,} templates")
    per = collections.Counter()
    for template, n in several.items():
        per[len(rows_of(template))] += n
    print("  by how many rows: " + ", ".join(f"{rows} rows {n:,}" for rows, n in sorted(per.items())))

    rows = collections.Counter()
    twice = []
    for template, n in several.items():
        each = rows_of(template)
        if len({row.lower() for row in each}) < len(each):
            twice.append(template)
        for row in each:
            rows[row] += n
    alone = {t.lower() for t in whole if "\n" not in t}
    also = [row for row in rows if row.lower() in alone]
    numbered = [row for row in rows if "#" in row]
    wrapped = [row for row in rows if row[:1].islower()]
    print(f"their rows: {len(rows):,} templates; {len(numbered):,} carry a number")
    print(f"  also a mod's whole text, which the vocabulary lists: {len(also):,}")
    print(f"  beginning lower-case, a sentence wrapped: {len(wrapped):,} templates in {sum(rows[r] for r in wrapped):,} lines")
    print(f"  a mod displaying one template in two rows: {len(twice)}")

    print(f"rows a total's row names ({table}):")
    found = 0
    for template, n in sorted(several.items(), key=lambda kv: -kv[1]):
        for row in rows_of(template):
            if row.lower() in totals:
                found += 1
                print(f"  {n:,} lines: {row!r} in {template!r}")
                print(f"      under {', '.join(sorted(set(totals[row.lower()])))}")
    if not found:
        print("  none")

    with open(STATS, encoding="utf-8") as f:
        stats = json.load(f)["result"]
    texts = [entry["text"] for group in stats for entry in group["entries"]]
    of_several = [t for t in texts if "\n" in t]
    distinct = set(of_several)
    print(f"the trade site: {len(texts):,} stats, {len(of_several):,} of several rows, {len(distinct):,} distinct texts")
    print(f"  a text displaying one row twice: {sum(1 for t in distinct if len({r.lower() for r in t.split(chr(10))}) < len(t.split(chr(10))))}")
    site_rows = {row.lower().replace("+#", "#") for t in distinct for row in t.split("\n")}
    print(f"  rows of them a total's row names: {len(site_rows & set(totals))}")
    # a stat whose rows sit apart in a mod of the copy: every row of the
    # stat is a row of the mod, in order, and the stat's text is no run of it
    apart = collections.Counter()
    folded = {t.lower().replace("+#", "#"): t for t in distinct}
    for template, n in several.items():
        mod = [row.lower() for row in rows_of(template)]
        for text, shown in folded.items():
            stat = text.split("\n")
            if len(stat) >= len(mod) or not all(row in mod for row in stat):
                continue
            at = [mod.index(row) for row in stat]
            if at == sorted(at) and at[-1] - at[0] + 1 != len(stat):
                apart[(template, shown)] += n
    print(f"mods of the copy holding a stat of the site's with another's row between its rows: {len(apart)}")
    for (template, shown), n in sorted(apart.items(), key=lambda kv: -kv[1]):
        print(f"  {n:,} lines: {shown!r} in {template!r}")


main()
