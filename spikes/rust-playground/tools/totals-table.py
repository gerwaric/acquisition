#!/usr/bin/env python3
"""The totals table (C94, C68): every named total the search answers — a
reviewed sum over an item's lines, defined once — regenerated from the C++
app's pseudomod tables and from what the trade site itself answered into the
reviewed file the search crate ships.

    python3 tools/totals-table.py            # writes crates/acquisition-search/reference/totals-v6.toml
    python3 tools/totals-table.py --check    # exits 1 when the file on disk differs from what the sources give

Inputs (read-only, committed):
  search/cpp-search/data/pseudomods.toml     the 35 tables of src/pseudomods.cpp at acquisition-qt@946a4f51
                                             (the app's own published definitions, since 2016; the
                                             file's header commit is the pin, and its first line names it)
  search/pseudo-stats/data/pseudo-classes.csv the trade site's pseudo stat each table's text names
                                             (`sum` rows whose evidence is pseudomods.toml, plus the one
                                             the site answered itself: q5, total fire resistance)
  search/pseudo-stats/data/table-changes.csv what the site's own answers ask of the table, one change a
                                             row, each with the captures that ask for it (the build plan,
                                             steps 9c to 9d-C: the searches the owner ran, 2026-09-26
                                             to 2026-10-08; the track's README). Every total counts
                                             what the site's pseudo of that
                                             name counts (owner, V6), and where the owner ruled the site
                                             wrong the row says `not mimicked` and the line stays counted

Rules, each a reading of the source and never a judgment (C106):
  1. A table becomes one total; its name is the builder's (the plan, rule 4):
     `total_res` as the worked example writes it, and the rest in that style, in
     NAMES below — a name is never derived from the site's id, which is
     carried on the row as `site` for the translation to read (C99).
  2. A listed template becomes a row: the crate's template form — a `+`
     before a `#` is spelling and is dropped (`parse.rs`, H2) —
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
  8. A change `no sum` becomes a reading, written apart from the totals: a
     pseudo that is a reading of other totals is no total and no row of one
     (C101). It carries the name the change names, the site's id and text,
     `kind` — the reading the change's `template` states, `count` (how many
     of the totals the item shows) or `least` (the smallest of them, where
     it shows every one) — and `of`, the totals read, each one this table
     writes. What a reading is from there is `src/pseudo.rs` (C101), said
     once. A total with a change `limit` the owner has not ruled on waits,
     counted in the header; one he has ruled on is written with its
     `limit`, which its definition prints: how far under the site's a
     line's text may be (owner, 2026-09-27: "yes, let's go with what we can
     observe directly from the text we have.").
  7. A change `new total` that reads the average of a ranged line becomes a
     ranged total (`ranged = true`): the ranged family, the 21 named
     `adds_…` (the build plan, 9d-C). Its rows name no slot — a ranged
     total sums low with low and high with high — and each is the change's:
     an `add row` that means the line whatever the item is, and a `twin`,
     a row that means one of the two stats the unsuffixed `Adds # to # <type>
     Damage` displays. The one twin a row can mean is `global`, the stat
     that is no weapon's own: written `not_on = "weapon"`, the category id
     the line is not counted on (J7: on a weapon the line is the weapon's
     own). What a weapon is, is the category table's answer and never this
     file's; any other twin stops the run. Every ranged total carries the
     limit its definition prints, RANGED_LIMIT below: a line under the other
     stat on a weapon is read as the weapon's own, with what the site showed.
  9. The change `rule` that a total leaves out arrays (J29; the owner,
     2026-10-08: "Leave both out.") is written once, in the table's header
     and on no row: `leaves_out`, the arrays the rule names, and
     `leaves_out_rule`, the rule's own sentence, which every total's
     definition prints. No rule of that form, or two, stops the run.
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
OUT = os.path.join(ROOT, "crates", "acquisition-search", "reference", "totals-v6.toml")
VERSION = 6
REALMS = ["pc", "xbox", "sony"]  # the C++ app searched PoE1; poe2 has no table yet
PIN = "acquisition-qt@946a4f51"

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

# rule 8: a reading as table-changes.csv states it, `the least of a, b, c`
READING = re.compile(r"^the (count|least) of ([a-z0-9_]+(?:, [a-z0-9_]+)+)$")

# rule 7: the twin a row can mean, as the category id its line is not counted on
TWINS = {"global": "weapon"}
RANGED_LIMIT = ("a line under the other stat on a weapon is read as the weapon's own: no weapon the "
                "trade site listed was found to carry one (round i, i06–i08)")

# rule 9: the rule that names the arrays no total reads
LEAVES_OUT = re.compile(r"^a total leaves out .*: the arrays ((?:`\w+Mods`(?:, | and )?)+), and no other array$")

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


def leaves_out(changes):
    """Rule 9: the arrays the rule names, and the rule's own sentence."""
    rules = [m for c in changes if c["change"] == "rule" and (m := LEAVES_OUT.match(c["template"]))]
    if len(rules) != 1:
        sys.exit(f"{len(rules)} rules name the arrays a total leaves out, and rule 9 writes one")
    return re.findall(r"`(\w+Mods)`", rules[0].group(1)), rules[0].group(0)


def apply(totals, changes):
    """Rules 4 to 8. Returns the rows added, what waits by kind, the limits, the readings and the
    ranged totals by name."""
    by_name = {t[0]: t for t in totals}
    waits = {"limit": {c["total"] for c in changes
                       if c["change"] == "limit" and c["twin_or_reads"] != "ruled"}}
    ranged = set()
    no_sum = {c["total"] for c in changes if c["change"] == "no sum"}
    added = 0
    for c in changes:
        if c["change"] == "new total":
            if c["total"] in waits["limit"]:
                continue
            if c["twin_or_reads"] == "avg":
                ranged.add(c["total"])
            elif c["twin_or_reads"] != "slot":
                sys.exit(f"{c['total']!r}: a new total reads `slot` or `avg`, never {c['twin_or_reads']!r}")
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
        if c["total"] in waits["limit"] or c["total"] in no_sum:
            continue
        if c["change"] == "remove row":
            sys.exit(f"{c['total']!r}: a row removed — a reviewed change, and rule 6 has no case for it")
        if c["change"] not in ("add row", "twin"):
            continue
        rows = by_name[c["total"]][3]
        if any(r[0] == c["template"] for r in rows):
            sys.exit(f"{c['total']!r}: {c['template']!r} is a row already")
        if c["total"] in ranged:
            # rule 7: a ranged line, no slot, a whole weight, and the twin it means
            if c["template"].count("#") != 2 or c["template"].count("# to #") != 1:
                sys.exit(f"{c['total']!r}: {c['template']!r} is no ranged line, and rule 7 sums low and high")
            weight = weight_of(c["weight"])
            if not isinstance(weight, int):
                sys.exit(f"{c['total']!r}: {c['template']!r} at a half, and a ranged total's `avg` is a mean")
            if c["change"] == "twin":
                if c["twin_or_reads"] not in TWINS:
                    sys.exit(f"{c['total']!r}: {c['template']!r} means the twin {c['twin_or_reads']!r}, "
                             "which no row can say (rule 7)")
                rows.append([c["template"], weight, TWINS[c["twin_or_reads"]]])
            elif c["twin_or_reads"] != "any":
                sys.exit(f"{c['total']!r}: {c['template']!r} is added as {c['twin_or_reads']!r}, never a twin")
            else:
                rows.append([c["template"], weight, None])
            added += 1
            continue
        if c["change"] == "twin":
            sys.exit(f"{c['total']!r}: a row that means a twin on a total that reads a line's number")
        if c["template"].count("#") != 1:
            sys.exit(f"{c['total']!r}: {c['template']!r} has {c['template'].count('#')} numbers, and rule 4 names arg1")
        rows.append([c["template"], weight_of(c["weight"])])
        added += 1
    for name in ranged:
        if not by_name[name][3]:
            sys.exit(f"{name!r}: a ranged total with no row")
    readings = []
    for c in changes:
        if c["change"] != "no sum":
            continue
        stated = READING.match(c["template"])
        if not stated:
            sys.exit(f"{c['total']!r}: {c['template']!r} is no reading rule 8 knows")
        kind, of = stated.group(1), stated.group(2).split(", ")
        if c["total"] in by_name or any(r[0] == c["total"] for r in readings):
            sys.exit(f"{c['total']!r}: a reading named as a total or another reading is")
        for name in of:
            if name not in by_name or name in waits["limit"] or name in ranged:
                sys.exit(f"{c['total']!r}: it reads {name!r}, which this table does not write")
        if len(set(of)) != len(of):
            sys.exit(f"{c['total']!r}: it reads a total twice")
        readings.append((c["total"], c["site_id"].removeprefix("pseudo."), c["site_text"], kind, of))
    return added, waits, limits, readings, ranged


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
    changes = read_changes()
    added, waits, limits, readings, ranged = apply(totals, changes)
    arrays, rule = leaves_out(changes)
    ranged_rows = sum(len(t[3]) for t in totals if t[0] in ranged)
    twins = sum(1 for t in totals if t[0] in ranged for r in t[3] if r[2])
    names = [t[0] for t in totals] + [r[0] for r in readings]
    if len(set(names)) != len(names):
        sys.exit("two tables got one name")
    n_rows = sum(len(t[3]) for t in totals)
    weighted = sum(1 for t in totals for r in t[3] if r[1] > 1)
    halves = sum(1 for t in totals for r in t[3] if r[1] == 0.5)
    below = sum(1 for t in totals for r in t[3] if r[1] < 0)

    out = [
        "# The totals table — v6 (C94, C101, C68; decisions/search.md), item search steps 7 and 9c",
        "# to 9d-C.",
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
        "# diff at each change (the admission test, the crate's doc, C106): a",
        "# convention with a definition — never a judgment. Its rules are the",
        "# script's docstring.",
        "#",
        "# What a row means: each of the item's lines showing the `template` — from",
        "# any source and under any flag unless the row names a `source` or `flag` —",
        "# contributes that line's `slot` times the row's `weight`; a row with no",
        "# slot contributes its weight per line. What the total is from there is",
        "# `src/totals.rs` (C94), said once.",
        "#",
        "# What no row reads: a line in an array `leaves_out` names — a crucible",
        "# node's lines and a flask's own, which the trade site files under no stat",
        "# and counts toward no pseudo (J29; the owner, 2026-10-08: \"Leave both",
        "# out.\"). Said here once, for every total, and on no row; `leaves_out_rule`",
        "# is the rule as search/pseudo-stats/data/table-changes.csv states it, which",
        "# every total's definition prints.",
        "#",
        "# What a ranged total means (`ranged = true`): its rows are ranged lines and",
        "# name no slot — low is summed with low and high with high. A row with",
        "# `not_on` means the one of two stats displaying its text that is no",
        "# weapon's own: it counts its line only on an item the category id does not",
        "# hold on, which is the category table's answer (`src/category.rs`) and",
        "# never this file's (J7, J30).",
        "#",
        "# What a reading means: a pseudo of the site's that is no sum of lines, and",
        "# so no total and no row of one (C101) — `count`, how many of the totals",
        "# `of` names the item shows; `least`, the smallest of them, where the item",
        "# shows every one. What the reading is from there is `src/pseudo.rs`",
        "# (C101), said once.",
        "#",
        f"# Measured at generation: {len(totals)} totals, {n_rows} rows — {listed} the C++ tables",
        f"# list, {added} the site's answers add — {weighted} with a weight above 1, {halves} at a",
        f"# half and {below} below nothing, a row's `reduced` spelling. Of them the ranged",
        f"# family: {len(ranged)} totals, {ranged_rows} rows, {twins} of which mean one of two stats",
        "# displaying one text and read what the item is.",
        f"# Written apart, as readings: {len(readings)} pseudos that are a reading of other totals",
        f"# and no sum of lines. Not written: {len(waits['limit'])} whose limit the owner has not",
        f"# ruled on. Written with a limit its definition prints: {len(limits)}.",
        "",
        f"version = {VERSION}",
        f"realms = [{', '.join(toml_str(r) for r in REALMS)}]",
        f"source = {toml_str(f'the trade site’s pseudo stats, by what the site answered of each (search/pseudo-stats/data/table-changes.csv), over the C++ app’s pseudomod tables, src/pseudomods.cpp at {PIN}')}",
        'generated_by = "tools/totals-table.py"',
        f"leaves_out = [{', '.join(toml_str(a) for a in arrays)}]",
        f"leaves_out_rule = {toml_str(rule)}",
    ]
    for name, site, text, rows in totals:
        out.append("")
        out.append("[[total]]")
        out.append(f"name = {toml_str(name)}")
        out.append(f"site = {toml_str(site)}")
        out.append(f"text = {toml_str(text)}")
        if name in ranged:
            if name in limits:
                sys.exit(f"{name!r}: a ranged total with a limit of its own beside rule 7's")
            out.append("ranged = true")
            out.append("limit = " + toml_str(RANGED_LIMIT))
            out.append("rows = [")
            for template, weight, not_on in rows:
                reads = f", not_on = {toml_str(not_on)}" if not_on else ""
                out.append(f"  {{ template = {toml_str(template)}, weight = {weight}{reads} }},")
            out.append("]")
            continue
        if name in limits:
            out.append("limit = " + toml_str(
                "read from the item's text, which cuts what the trade site rounds: under the "
                f"site's by {limits[name]} a line at most"))
        out.append("rows = [")
        for template, weight in rows:
            out.append(f'  {{ template = {toml_str(template)}, slot = "arg1", weight = {weight} }},')
        out.append("]")
    for name, site, text, kind, of in readings:
        out.append("")
        out.append("[[reading]]")
        out.append(f"name = {toml_str(name)}")
        out.append(f"site = {toml_str(site)}")
        out.append(f"text = {toml_str(text)}")
        out.append(f"kind = {toml_str(kind)}")
        out.append(f"of = [{', '.join(toml_str(n) for n in of)}]")
    text = "\n".join(out) + "\n"

    if "--check" in sys.argv:
        with open(OUT, encoding="utf-8") as f:
            if f.read() != text:
                sys.exit(f"{OUT} differs from what the sources give: regenerate and review the diff")
        print(f"ok      {os.path.relpath(OUT, ROOT)} is what the sources give")
        return
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print(f"wrote {os.path.relpath(OUT, ROOT)}: {len(totals)} totals, {n_rows} rows, {len(readings)} readings")


main()
