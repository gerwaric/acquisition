#!/usr/bin/env python3
"""The totals table (C94, C68): every named total the search answers — a
reviewed sum over an item's lines, defined once — regenerated from the C++
app's pseudomod tables into the reviewed file the search crate ships.

    python3 tools/totals-table.py            # writes crates/acquisition-search/reference/totals-v1.toml
    python3 tools/totals-table.py --check    # exits 1 when the file on disk differs from what the source gives

Inputs (read-only, committed):
  search/cpp-search/data/pseudomods.toml     the 35 tables of src/pseudomods.cpp at master@946a4f51
                                             (the app's own published definitions, since 2016; the
                                             file's header commit is the pin, and its first line names it)
  search/pseudo-stats/data/pseudo-classes.csv the trade site's pseudo stat each table's text names
                                             (`sum` rows whose evidence is pseudomods.toml, plus the one
                                             the site answered itself: q5, total fire resistance)

Rules, each a reading of the source and never a judgment (C106):
  1. A table becomes one total; its name is the builder's (the plan, rule 4):
     `total_res` as the reference writes it, and the rest in that style, in
     NAMES below — a name is never derived from the site's id, which is
     carried on the row as `site` for the translation to read (C99).
  2. A listed template becomes a row: the crate's template form — a `+`
     before a `#` is spelling and is dropped (the reference, *Strings*, H2) —
     with `slot = "arg1"` (every listed template has one number) and its
     weight the number of times the source lists it: the all-elemental line
     three times in `+#% total Elemental Resistance` is one row, weight 3
     (C94's own example).
  3. No source or flag on a row: the source's rule counts any bucket
     (pseudomods.toml's header), so a row admits every source and every
     flag.
Nothing is added, renamed beyond rule 1, merged beyond rule 2, or dropped;
the counts the header prints are measured at generation. The site's own
answer on ten fetched items (`search/trade-query/data/fetch-census.json`,
q5) agrees with the fire-resistance table on every one, worked by hand
2026-09-23; the coverage trial the plan owes (M6) is
search/pseudo-stats/scripts/coverage-trial.py.
"""

import csv
import os
import re
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
SOURCE = os.path.join(ROOT, "search", "cpp-search", "data", "pseudomods.toml")
CLASSES = os.path.join(ROOT, "search", "pseudo-stats", "data", "pseudo-classes.csv")
OUT = os.path.join(ROOT, "crates", "acquisition-search", "reference", "totals-v1.toml")
VERSION = 1
REALMS = ["pc", "xbox", "sony"]  # the C++ app searched PoE1; poe2 has no table yet
PIN = "master@946a4f51"

# rule 1: the builder's names, one per table of the source
NAMES = {
    "+#% total to Cold Resistance": "total_cold_res",
    "+#% total to Fire Resistance": "total_fire_res",
    "+#% total to Lightning Resistance": "total_lightning_res",
    "+#% total Elemental Resistance": "total_ele_res",
    "+#% total to Chaos Resistance": "total_chaos_res",
    "+#% total Resistance": "total_res",
    "+# total to Strength": "total_str",
    "+# total to Dexterity": "total_dex",
    "+# total to Intelligence": "total_int",
    "+#% total Attack Speed": "total_attack_speed",
    "+#% total Cast Speed": "total_cast_speed",
    "+#% total increased Physical Damage": "total_increased_phys",
    "+#% total Critical Strike Chance for Spells": "total_spell_crit",
}
GEM_LEVELS = re.compile(r"^\+# total to Level of Socketed (?:(\w+) )?Gems$")

# rule 2: a `+` straight before a `#` is spelling (template.rs, `unsigned`)
PLUS = re.compile(r"(?<![#)\d])\+(?=#)")


def name_of(text):
    if text in NAMES:
        return NAMES[text]
    m = GEM_LEVELS.match(text)
    if m:
        tag = m.group(1)
        return f"total_{tag.lower()}_gem_levels" if tag else "total_gem_levels"
    sys.exit(f"no name for the table {text!r}: add it to NAMES")


def read_source():
    """pseudomods.toml is small and regular: [[pseudomod]] blocks, `name` and a `sums` list."""
    with open(SOURCE, encoding="utf-8") as f:
        text = f.read()
    if PIN not in text.splitlines()[0]:
        sys.exit(f"{SOURCE} is not the extract of {PIN}: the pin here is a reviewed change")
    tables = []
    for block in text.split("[[pseudomod]]")[1:]:
        name = re.search(r'^name = "(.*)"$', block, re.M).group(1)
        sums = re.findall(r'^\s+"(.*)",$', block, re.M)
        tables.append((name, sums))
    return tables


def read_sites():
    sites = {}
    with open(CLASSES, encoding="utf-8", newline="\n") as f:
        for row in csv.DictReader(f):
            if row["class"] == "sum":
                sites[row["text"]] = row["id"].removeprefix("pseudo.")
    return sites


def toml_str(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def main():
    tables = read_source()
    sites = read_sites()
    totals = []
    for text, sums in tables:
        rows = []
        for template in sums:
            template = PLUS.sub("", template)
            if template.count("#") != 1:
                sys.exit(f"{text!r}: {template!r} has {template.count('#')} numbers, and rule 2 names arg1")
            for row in rows:
                if row[0] == template:
                    row[1] += 1
                    break
            else:
                rows.append([template, 1])
        site = sites.get(text) or sites.get(text.replace("+#% total", "#% total"))
        if site is None:
            sys.exit(f"{text!r}: the site names no pseudo stat for it in {CLASSES}")
        totals.append((name_of(text), site, text, rows))
    names = [t[0] for t in totals]
    if len(set(names)) != len(names):
        sys.exit("two tables got one name")
    n_rows = sum(len(t[3]) for t in totals)
    weighted = sum(1 for t in totals for r in t[3] if r[1] > 1)

    out = [
        "# The totals table — v1 (C94, C68; decisions/search.md), item search step 7.",
        "#",
        "# Reference data: reviewed, committed, shipped inside the binary",
        "# (`include_str!` in `src/totals.rs`), read-only, never in a store file,",
        "# enumerable by `acq search --describe computed`, cited by version in every",
        "# basis (C98). Generated by tools/totals-table.py from the C++ app's",
        f"# pseudomod tables ({PIN}, src/pseudomods.cpp; the extract",
        "# search/cpp-search/data/pseudomods.toml), the app's own published",
        "# definitions, which name the trade site's pseudo stats (search/pseudo-stats/).",
        "# Reviewed as a diff at each change (the admission test, search/DESIGN.md,",
        "# C106): a convention with a published definition — never a judgment. Its",
        "# rules are the script's docstring.",
        "#",
        "# What a total means (C94): for each of the item's lines that a row names",
        "# — the template, from any source and under any flag unless the row says",
        "# `source` or `flag` — the row contributes that line's `slot` times its",
        "# `weight`; a row with no slot contributes its weight per line. The total",
        "# is the sum of the contributions, exact in decimals; a line the item lacks",
        "# adds nothing, and a total of nothing is zero. A contributor the search",
        "# could not read leaves a subtotal marked incomplete, never a total; an item",
        "# in a realm this table does not cover has no total, never zero.",
        "#",
        f"# Measured at generation: {len(totals)} totals, {n_rows} rows, {weighted} rows with a",
        "# weight above 1 (a template the source lists more than once).",
        "",
        f"version = {VERSION}",
        f"realms = [{', '.join(toml_str(r) for r in REALMS)}]",
        f"source = {toml_str(f'the C++ app’s pseudomod tables, src/pseudomods.cpp at {PIN} (35 tables, one per trade-site pseudo stat); search/cpp-search/data/pseudomods.toml')}",
        'generated_by = "tools/totals-table.py"',
    ]
    for name, site, text, rows in totals:
        out.append("")
        out.append("[[total]]")
        out.append(f"name = {toml_str(name)}")
        out.append(f"site = {toml_str(site)}")
        out.append(f"text = {toml_str(text)}")
        out.append("rows = [")
        for template, weight in rows:
            out.append(f'  {{ template = {toml_str(template)}, slot = "arg1", weight = {weight} }},')
        out.append("]")
    text = "\n".join(out) + "\n"

    if "--check" in sys.argv:
        with open(OUT, encoding="utf-8") as f:
            if f.read() != text:
                sys.exit(f"{OUT} differs from what {SOURCE} gives: regenerate and review the diff")
        print(f"ok      {os.path.relpath(OUT, ROOT)} is what the source gives")
        return
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print(f"wrote {os.path.relpath(OUT, ROOT)}: {len(totals)} totals, {n_rows} rows")


main()
