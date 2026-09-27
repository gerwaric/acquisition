#!/usr/bin/env python3
"""The totals table (C94, C68): every named total the search answers — a
reviewed sum over an item's lines, defined once — regenerated from the C++
app's pseudomod tables and from what the trade site itself answered into the
reviewed file the search crate ships.

    python3 tools/totals-table.py            # writes crates/acquisition-search/reference/totals-v4.toml
    python3 tools/totals-table.py --check    # exits 1 when the file on disk differs from what the sources give

Inputs (read-only, committed):
  search/cpp-search/data/pseudomods.toml     the 35 tables of src/pseudomods.cpp at master@946a4f51
                                             (the app's own published definitions, since 2016; the
                                             file's header commit is the pin, and its first line names it)
  search/pseudo-stats/data/pseudo-classes.csv the trade site's pseudo stat each table's text names
                                             (`sum` rows whose evidence is pseudomods.toml, plus the one
                                             the site answered itself: q5, total fire resistance)
  search/pseudo-stats/data/table-changes.csv what the site's own answers ask of the table, one change a
                                             row, each with the captures that ask for it (the build plan,
                                             steps 9c and 9c2: the searches the owner ran, 2026-09-26
                                             and 2026-09-27; the track's README). Every total counts
                                             what the site's pseudo of that
                                             name counts (owner, V6), and where the owner ruled the site
                                             wrong the row says `not mimicked` and the line stays counted

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
  4. A change `add row` that means any stat displaying its text becomes a
     row of its total, `slot = "arg1"`, at the weight the change names: a
     line's two eldritch forms, `All Resistances`, and at a weight below
     nothing a row's `reduced` spelling (owner, 2026-09-27: "yes, we need to
     be able to find reduced lines and totals").
  5. A change `new total` that reads a line's number becomes a total, named
     as the change names it, with the site's id and text and its own rows:
     `total_life`.
  6. A change `not mimicked` changes nothing: what the site leaves out and
     the owner ruled counted stays as the source lists it, or is counted by
     its text as every line is. A change `rule` is `totals.rs`'s, or a row
     of every total it names (rule 4).
  8. A change `no sum` writes nothing: a pseudo that is a reading of other
     totals is no total, and what it is built as is not this table's. A
     total with a change `limit` the owner has not ruled on waits, counted
     in the header; one he has ruled on is written with its `limit`, which
     its definition prints: how far under the site's a line's text may be
     (owner, 2026-09-27: "yes, let's go with what we can observe directly
     from the text we have.").
  7. What waits, counted in the header and not built: a change `twin`, a row
     that means one of two stats displaying one text, which a table row
     cannot say until it can name what the item is; and with it every total
     that reads the average of a ranged line — the ranged family, all of
     whose totals hold such a row or are an aggregate of those that do.
Nothing else is added, renamed beyond rule 1, merged beyond rule 2, or dropped;
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
CHANGES = os.path.join(ROOT, "search", "pseudo-stats", "data", "table-changes.csv")
OUT = os.path.join(ROOT, "crates", "acquisition-search", "reference", "totals-v4.toml")
VERSION = 4
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


def read_changes():
    with open(CHANGES, encoding="utf-8", newline="\n") as f:
        return list(csv.DictReader(f))


def weight_of(text):
    """A weight as the table writes it: a whole number or a half, above nothing or below."""
    value = float(text)
    if value * 2 != int(value * 2) or value == 0:
        sys.exit(f"a weight is a whole number or a half, never {text}")
    return int(value) if value == int(value) else value


def apply(totals, changes):
    """Rules 4 to 8. Returns the totals and what waits, by kind."""
    by_name = {t[0]: t for t in totals}
    waits = {"twin": 0, "ranged": set(),
             "limit": {c["total"] for c in changes
                       if c["change"] == "limit" and c["twin_or_reads"] != "ruled"},
             "no sum": {c["total"] for c in changes if c["change"] == "no sum"}}
    added = 0
    for c in changes:
        if c["change"] == "new total":
            if c["total"] in waits["limit"]:
                continue
            if c["twin_or_reads"] != "slot":
                waits["ranged"].add(c["total"])
                continue
            if c["total"] in by_name:
                sys.exit(f"{c['total']!r}: a new total the source already has")
            total = (c["total"], c["site_id"].removeprefix("pseudo."), c["site_text"], [])
            totals.append(total)
            by_name[c["total"]] = total
    limits = {c["total"]: c["weight"] for c in changes
              if c["change"] == "limit" and c["twin_or_reads"] == "ruled"}
    for name in limits:
        if name not in by_name:
            sys.exit(f"{name!r}: a limit on a total the table does not write")
    for c in changes:
        if c["total"] in waits["ranged"]:
            waits["twin"] += c["change"] == "twin"
            continue
        if c["total"] in waits["limit"] or c["total"] in waits["no sum"]:
            continue
        if c["change"] == "twin":
            sys.exit(f"{c['total']!r}: a row that means a twin on a total that reads a line's number")
        if c["change"] == "remove row":
            sys.exit(f"{c['total']!r}: a row removed — a reviewed change, and rule 6 has no case for it")
        if c["change"] != "add row":
            continue
        rows = by_name[c["total"]][3]
        if c["template"].count("#") != 1:
            sys.exit(f"{c['total']!r}: {c['template']!r} has {c['template'].count('#')} numbers, and rule 4 names arg1")
        if any(r[0] == c["template"] for r in rows):
            sys.exit(f"{c['total']!r}: {c['template']!r} is a row already")
        rows.append([c["template"], weight_of(c["weight"])])
        added += 1
    return added, waits, limits


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
    listed = sum(len(t[3]) for t in totals)
    added, waits, limits = apply(totals, read_changes())
    names = [t[0] for t in totals]
    if len(set(names)) != len(names):
        sys.exit("two tables got one name")
    n_rows = sum(len(t[3]) for t in totals)
    weighted = sum(1 for t in totals for r in t[3] if r[1] > 1)
    halves = sum(1 for t in totals for r in t[3] if r[1] == 0.5)
    below = sum(1 for t in totals for r in t[3] if r[1] < 0)

    out = [
        "# The totals table — v4 (C94, C68; decisions/search.md), item search steps 7, 9c and 9c2.",
        "#",
        "# Reference data: reviewed, committed, shipped inside the binary",
        "# (`include_str!` in `src/totals.rs`), read-only, never in a store file,",
        "# enumerable by `acq search --describe computed`, cited by version in every",
        "# basis (C98). Generated by tools/totals-table.py from the C++ app's",
        f"# pseudomod tables ({PIN}, src/pseudomods.cpp; the extract",
        "# search/cpp-search/data/pseudomods.toml), which name the trade site's pseudo",
        "# stats, and from what the site itself answered of each",
        "# (search/pseudo-stats/data/table-changes.csv: a change a row, each with the",
        "# captures that ask for it). Every total counts what the site's pseudo of",
        "# that name counts, but where the owner ruled the site wrong. Reviewed as a",
        "# diff at each change (the admission test, search/DESIGN.md, C106): a",
        "# convention with a definition — never a judgment. Its rules are the",
        "# script's docstring.",
        "#",
        "# What a row means: each of the item's lines showing the `template` — from",
        "# any source and under any flag unless the row names a `source` or `flag` —",
        "# contributes that line's `slot` times the row's `weight`; a row with no",
        "# slot contributes its weight per line. What the total is from there is",
        "# `src/totals.rs` (C94), said once.",
        "#",
        f"# Measured at generation: {len(totals)} totals, {n_rows} rows — {listed} the C++ tables",
        f"# list, {added} the site's answers add — {weighted} with a weight above 1, {halves} at a",
        f"# half and {below} below nothing, a row's `reduced` spelling. Not built, waiting",
        f"# for a row that can name what the item is: {len(waits['ranged'])} totals of the ranged",
        f"# family, {waits['twin']} of whose rows mean one of two stats displaying one text.",
        f"# Not written: {len(waits['no sum'])} pseudos that are a reading of other totals and no sum of",
        f"# lines, and {len(waits['limit'])} whose limit the owner has not ruled on. Written with a",
        f"# limit its definition prints: {len(limits)}.",
        "",
        f"version = {VERSION}",
        f"realms = [{', '.join(toml_str(r) for r in REALMS)}]",
        f"source = {toml_str(f'the trade site’s pseudo stats, by what the site answered of each (search/pseudo-stats/data/table-changes.csv), over the C++ app’s pseudomod tables, src/pseudomods.cpp at {PIN}')}",
        'generated_by = "tools/totals-table.py"',
    ]
    for name, site, text, rows in totals:
        out.append("")
        out.append("[[total]]")
        out.append(f"name = {toml_str(name)}")
        out.append(f"site = {toml_str(site)}")
        out.append(f"text = {toml_str(text)}")
        if name in limits:
            out.append("limit = " + toml_str(
                "read from the item's text, which cuts what the trade site rounds: under the "
                f"site's by {limits[name]} a line at most"))
        out.append("rows = [")
        for template, weight in rows:
            out.append(f'  {{ template = {toml_str(template)}, slot = "arg1", weight = {weight} }},')
        out.append("]")
    text = "\n".join(out) + "\n"

    if "--check" in sys.argv:
        with open(OUT, encoding="utf-8") as f:
            if f.read() != text:
                sys.exit(f"{OUT} differs from what the sources give: regenerate and review the diff")
        print(f"ok      {os.path.relpath(OUT, ROOT)} is what the sources give")
        return
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print(f"wrote {os.path.relpath(OUT, ROOT)}: {len(totals)} totals, {n_rows} rows")


main()
