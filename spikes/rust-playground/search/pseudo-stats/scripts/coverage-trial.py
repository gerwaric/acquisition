#!/usr/bin/env python3
"""M6 of search/BUILD-PLAN.md, the totals coverage trial (the park in
decisions/search.md, fired at step 7): on the real corpus, how many lines
whose text holds a word do the named totals not count?

    cargo build --workspace && python3 search/item-facts/scripts/m2-differential.py   # writes raw/m2/rust.json
    python3 search/pseudo-stats/scripts/coverage-trial.py [WORD ...]

Input: the deriver's own census of the owner's store copy
(search/item-facts/raw/m2/rust.json — a row per (source, template) with
its items and lines, in the crate's template form), and the shipped table
(crates/acquisition-search/reference/totals-v1.toml). For each word
(default: resist, strength, dexterity, intelligence, attack speed, cast
speed, level of socketed) every template holding it is listed with its
lines, and marked with the totals whose rows name it; the rest is what no
total counts, ranked by lines. Lines, not items: the census counts items
per (source, template), so summed over sources and templates an item is
counted once per incidence — the second column says so (outside audit,
2026-09-24). A word-based trial measures coverage, not membership: a line
the word finds and no total counts is a candidate for the owner to rule
on, never a row added here. Never in the gate: the input is raw/.
"""
import collections
import json
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
CENSUS = os.path.join(ROOT, "search", "item-facts", "raw", "m2", "rust.json")
TABLE = os.path.join(ROOT, "crates", "acquisition-search", "reference", "totals-v1.toml")
WORDS = ["resist", "strength", "dexterity", "intelligence", "attack speed", "cast speed", "level of socketed"]


def read_table():
    """totals-v1.toml is regular: [[total]] blocks with `name` and inline row tables."""
    with open(TABLE, encoding="utf-8") as f:
        text = f.read()
    by_template = collections.defaultdict(list)
    for block in text.split("[[total]]")[1:]:
        name = re.search(r'^name = "(.*)"$', block, re.M).group(1)
        for template in re.findall(r'\{ template = "((?:[^"\\]|\\.)*)"', block):
            by_template[template.replace('\\"', '"').replace("\\\\", "\\")].append(name)
    return by_template


def main():
    words = [w.lower() for w in sys.argv[1:]] or WORDS
    if not os.path.exists(CENSUS):
        sys.exit(f"{CENSUS} is missing: run search/item-facts/scripts/m2-differential.py after cargo build")
    with open(CENSUS, encoding="utf-8") as f:
        census = json.load(f)
    counted_by = read_table()
    # by template over every source: a total admits any source
    templates = collections.defaultdict(lambda: [0, 0, set()])
    for row in census["rows"]:
        t = templates[row["template"]]
        t[0] += row["items"]
        t[1] += row["lines"]
        t[2].add(row["source"])
    print(f"corpus: {census['items']} items, {len(templates)} templates ({os.path.relpath(CENSUS, ROOT)})")
    print(f"table:  {sum(1 for _ in counted_by)} templates named by a total ({os.path.relpath(TABLE, ROOT)})")
    for word in words:
        hits = {t: v for t, v in templates.items() if word in t.lower()}
        counted = {t: v for t, v in hits.items() if t in counted_by}
        missed = {t: v for t, v in hits.items() if t not in counted_by}
        print()
        print(f"== {word!r}: {len(hits)} templates, {sum(v[1] for v in hits.values())} lines ({sum(v[0] for v in hits.values())} item-source-template incidences)")
        print(f"   counted by a total: {len(counted)} templates, {sum(v[1] for v in counted.values())} lines ({sum(v[0] for v in counted.values())} incidences)")
        print(f"   counted by none:    {len(missed)} templates, {sum(v[1] for v in missed.values())} lines ({sum(v[0] for v in missed.values())} incidences)")
        print(f"   {'lines':>7} {'incid.':>7}  template  [sources]  (totals)")
        for t, (items, lines, sources) in sorted(hits.items(), key=lambda kv: (-kv[1][1], kv[0])):
            names = ", ".join(counted_by.get(t, [])) or "-"
            print(f"   {lines:>7} {items:>7}  {t!r}  [{', '.join(sorted(sources))}]  ({names})")
    # every row of the table against the corpus: a template no item carries
    # is a row the corpus cannot exercise, said here and not guessed
    unseen = sorted(t for t in counted_by if t not in templates)
    print()
    print(f"== rows of the table the corpus never carries: {len(unseen)}")
    for t in unseen:
        print(f"   {t!r}  ({', '.join(counted_by[t])})")


main()
