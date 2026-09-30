#!/usr/bin/env python3
"""The ranged family's unsuffixed lines on the owner's copy (open question 7):
writes ../data/ranged-census.csv and ../data/ranged-uniques.csv and prints
the counts.

    ranged-census.py

A line is in the census when its template (search/item-facts's rule, the
spelling of search/item-facts/data/mod-templates.csv) is one of the five
`Adds # to # <Type> Damage` with no scope after it. The site gives that
text two stats, the `(Local)` twin — a weapon's own damage — and the other,
and its pseudos count them apart (tools/trade_rows.py, `ranged_changes`,
round three's change to the plain pseudos); a private item shows the text
alone, so the build tells them apart by what the item is (the plan, 9d).

Inputs, read-only: the owner's store copy
(search/item-facts/raw/m3/store/mock/GERWARIC_7694.db, opened with
`mode=ro`), every live item, socketed ones included, poe2's tag markup
collapsed to the text it shows; the shipped class
table (crates/acquisition-search/reference/classes-v1.toml) by the item's
base; search/repoe/data/class-to-trade-category.csv, a class a weapon on
the site when its trade id is a `weapon.*`; and the export at e2bd511a
(poe1/data/mods.json, stats.json, stat_translations.json), refused at any
other commit.

ranged-census.csv: one row per (template, class, weapon, realm, rarity,
source array, flags), with lines and items.

ranged-uniques.csv: one row per line on a unique item: the unique's name
and base, its class, and what the export says of the mod behind the line.
The join is by the line's text, as search/repoe/scripts join text: every
mod of the generation types the line's array is made from (a unique's own
lines and its base's implicit: `unique`; a crucible node's:
`crucible_unique_tree`, `crucible_tree`; an enchant: `enchantment`) with a
row of its text whose template is the line's and whose ranges hold the
line's two numbers; of each such mod, the stats the export translates to
the line's text (`stat_translations.json`), and their `is_local`
(`stats.json`). Where the mods found say one thing, that is the row's;
where they say both, or none is found, the row says so and why — the count
of those is the finding's limit, not a guess. Beside it, what the site says
where a capture holds the same unique (data/captures.json, the sittings'
fetches): the stat its line was listed under — the `(Local)` twin or the
other (search/trade-query/data/stats-2026-09-12.json) — or `not captured`.
"""

import csv
import json
import re
import sqlite3
import subprocess
import sys
import tomllib
from collections import Counter, defaultdict
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
SEARCH = TRACK.parent
ROOT = SEARCH.parent
CLONES = SEARCH.parents[3]
POE1 = CLONES / "poe1"
POE1_COMMIT = "e2bd511a0133bbe6c1ab548ef1285cb99f3cf0e9"
STORE = SEARCH / "item-facts/raw/m3/store/mock/GERWARIC_7694.db"
TEMPLATES = SEARCH / "item-facts/data/mod-templates.csv"
CLASSES = ROOT / "crates/acquisition-search/reference/classes-v1.toml"
CATEGORIES = SEARCH / "repoe/data/class-to-trade-category.csv"
CENSUS_OUT = TRACK / "data" / "ranged-census.csv"
UNIQUES_OUT = TRACK / "data" / "ranged-uniques.csv"
CAPTURES = TRACK / "data" / "captures.json"
TRADE_STATS = SEARCH / "trade-query/data/stats-2026-09-12.json"
INPUTS = ("../item-facts/raw/m3/store/mock/GERWARIC_7694.db (read-only), "
          "crates/acquisition-search/reference/classes-v1.toml, ../repoe/data/class-to-trade-category.csv")
SITE_INPUTS = "data/captures.json, ../trade-query/data/stats-2026-09-12.json"
EXPORT = "poe1/data/mods.json, stats.json and stat_translations.json @ e2bd511a"

TYPES = ["Physical", "Lightning", "Cold", "Fire", "Chaos"]
FAMILY = {f"Adds # to # {t} Damage" for t in TYPES}
NUM = re.compile(r"(?<![\d#])[-+]?\d+(?:\.\d+)?")  # search/item-facts/scripts/census.py, template()
RANGE = re.compile(r"\((-?\d+(?:\.\d+)?)-(-?\d+(?:\.\d+)?)\)|(-?\d+(?:\.\d+)?)")
PLACEHOLDER = re.compile(r"\{\d+(?::[^}]*)?\}")
GENERATION = {
    "explicitMods": ("unique",), "implicitMods": ("unique",),
    "crucibleMods": ("crucible_unique_tree", "crucible_tree"), "enchantMods": ("enchantment",),
}


def pinned(path, commit):
    head = subprocess.run(["git", "-C", str(path), "rev-parse", "HEAD"], capture_output=True, text=True,
                          check=True).stdout.strip()
    if head != commit:
        sys.exit(f"{path} is at {head}, not {commit}: the manifest names that commit")


MARKUP = re.compile(r"\[([^\[\]|]*)(?:\|([^\[\]]*))?\]")  # poe2's `[Fire|Fire]`, collapsed to what shows


def template(text):
    return NUM.sub("#", MARKUP.sub(lambda m: m.group(2) or m.group(1), text))


def numbers(text):
    return [float(n) for n in NUM.findall(text)]


def mod_rows(text):
    """Each row of a mod's text: (its template, the (low, high) of each number)."""
    out = []
    for row in text.split("\n"):
        spans = []

        def one(m):
            if m.group(3) is not None:
                v = float(m.group(3))
                spans.append((v, v))
            else:
                spans.append((float(m.group(1)), float(m.group(2))))
            return "#"
        out.append((RANGE.sub(one, row).replace("+#", "#"), spans))
    return out


def census_rows():
    """The five templates' rows in mod-templates.csv, by line number (its first line is a comment)."""
    rows = {}
    with TEMPLATES.open(newline="\n") as fh:
        next(fh)
        reader = csv.DictReader(fh)
        for row in reader:
            if row["template"] in FAMILY:
                rows.setdefault(row["template"], []).append(f"{row['array']} L{reader.line_num + 1}")
    return rows


def main():
    pinned(POE1, POE1_COMMIT)
    if not STORE.exists():
        print(f"{STORE.relative_to(ROOT)} is absent: every count here is an open question")
        return 1
    spelled = census_rows()
    missing = FAMILY - set(spelled)
    if missing:
        sys.exit(f"mod-templates.csv does not spell {sorted(missing)}")

    classes = defaultdict(set)
    for c in tomllib.loads(CLASSES.read_text())["class"]:
        for b in c["bases"]:
            classes[b].add(c["name"])
    with CATEGORIES.open(newline="\n") as fh:
        next(fh)
        trade = {r["class_name"]: r["trade_id"] for r in csv.DictReader(fh) if r["class_name"]}

    stats = json.loads((POE1 / "data/stats.json").read_text())
    renders = defaultdict(set)  # template -> the stats the export translates to it
    for entry in json.loads((POE1 / "data/stat_translations.json").read_text()):
        for s in entry.get("English", []):
            t = PLACEHOLDER.sub("#", s["string"]).replace("+#", "#")
            if t in FAMILY:
                renders[t].update(entry["ids"])
    by_row = defaultdict(list)  # (generation, template) -> [(mod id, spans, local flags)]
    for mod_id, mod in json.loads((POE1 / "data/mods.json").read_text()).items():
        for t, spans in mod_rows(mod.get("text") or ""):
            if t not in FAMILY:
                continue
            behind = [s["id"] for s in mod["stats"] if s["id"] in renders[t]]
            flags = {bool(stats.get(s, {}).get("is_local")) for s in behind}
            by_row[(mod["generation_type"], t)].append((mod_id, spans, flags))

    def weapon_of(names):
        if not names:
            return "no class"
        ids = {trade.get(n, "") for n in names}
        if all(i.startswith("weapon.") for i in ids):
            return "weapon"
        if not any(i.startswith("weapon.") for i in ids):
            return "not a weapon"
        return "undecided: classes of both kinds"

    local_ids = {e["id"] for g in json.loads(TRADE_STATS.read_text())["result"] for e in g["entries"]
                 if e["text"].endswith(" (Local)")}
    site = defaultdict(set)  # (unique name, template) -> {"local", "global"} as the site listed it
    for capture in json.loads(CAPTURES.read_text())["searches"].values():
        for item in capture["fetched"] or []:
            if item.get("rarity") != "Unique":
                continue
            for array, lines in item["lines"].items():
                for line in lines:
                    t = template(line["description"])
                    if t in FAMILY and array != "pseudoMods":
                        stat = line["hash"].removeprefix("stat.")
                        site[(item["name"], t)].add("local" if stat in local_ids else "global")

    db = sqlite3.connect(f"file:{STORE}?mode=ro", uri=True)
    census = Counter()
    census_items = defaultdict(set)
    uniques = []
    for item_id, realm, rarity, body in db.execute(
            "select id, realm, rarity, json from items where removed_at is null order by id"):
        body = json.loads(body)
        base = body.get("baseType") or body.get("typeLine") or ""
        names = classes.get(base, set()) if realm != "poe2" else set()
        cls = "|".join(sorted(names)) or ("poe2: no table" if realm == "poe2" else "not in the class table")
        weapon = weapon_of(names) if realm != "poe2" else "no class"
        for array, entries in body.items():
            if not (array.endswith("Mods") and isinstance(entries, list)):
                continue
            for entry in entries:
                text = entry.get("description") if isinstance(entry, dict) else entry
                if not isinstance(text, str):
                    continue
                flags = ";".join(sorted(k for k, v in (entry.get("flags") or {}).items() if v)) \
                    if isinstance(entry, dict) else ""
                for row in text.split("\n"):
                    t = template(row)
                    if t not in FAMILY:
                        continue
                    key = (t, cls, weapon, realm, rarity or "", array, flags)
                    census[key] += 1
                    census_items[key].add(item_id)
                    if rarity != "Unique":
                        continue
                    lo, hi = numbers(row)[:2]
                    found = [(m, f) for g in GENERATION.get(array, ())
                             for m, spans, f in by_row[(g, t)]
                             if len(spans) >= 2 and spans[0][0] <= lo <= spans[0][1]
                             and spans[1][0] <= hi <= spans[1][1]]
                    says = set().union(*(f for _, f in found)) if found else set()
                    if array not in GENERATION:
                        outcome = f"cannot say: no generation type joins a line of {array}"
                    elif not found:
                        outcome = (f"cannot say: no {'/'.join(GENERATION[array])} mod's text holds the line "
                                   "at its numbers")
                    elif says == {True}:
                        outcome = "local"
                    elif says == {False}:
                        outcome = "global"
                    elif not says:
                        outcome = "cannot say: the mods found carry no stat the export translates to the text"
                    else:
                        outcome = "cannot say: the mods found say local and global"
                    listed = site.get((body.get("name", ""), t))
                    uniques.append([body.get("name", ""), base, cls, weapon, realm, array, row, outcome,
                                    "/".join(sorted(listed)) if listed else "not captured",
                                    len(found), ";".join(sorted(m for m, _ in found))])

    with CENSUS_OUT.open("w", newline="") as fh:
        fh.write(f"# generated by scripts/ranged-census.py from {INPUTS}; templates as "
                 f"../item-facts/data/mod-templates.csv spells them\n")
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(["template", "class", "weapon", "realm", "rarity", "array", "flags", "lines", "items"])
        for key in sorted(census, key=lambda k: (k[0], k[2], k[1], k[3], k[4], k[5], k[6])):
            w.writerow(list(key) + [census[key], len(census_items[key])])
    uniques.sort(key=lambda r: (r[3], r[0], r[1], r[6]))
    with UNIQUES_OUT.open("w", newline="") as fh:
        fh.write(f"# generated by scripts/ranged-census.py from {INPUTS}, {SITE_INPUTS} and {EXPORT}\n")
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(["unique", "base", "class", "weapon", "realm", "array", "line", "export_says", "site_says",
                    "mods_found", "mods"])
        w.writerows(uniques)

    print("the five templates in mod-templates.csv: " + "; ".join(
        f"{t}: {', '.join(spelled[t])}" for t in sorted(spelled)))
    total = sum(census.values())
    by_weapon = Counter()
    for key, n in census.items():
        by_weapon[key[2]] += n
    print(f"{total} live lines on the copy, on {len(set().union(*census_items.values()))} items")
    for w_, n in sorted(by_weapon.items(), key=lambda kv: -kv[1]):
        print(f"  {n:>5}  {w_}")
    print("by template and weapon:")
    grid = Counter()
    for key, n in census.items():
        grid[(key[0], key[2])] += n
    for t in sorted(FAMILY):
        print(f"  {t:32} " + "  ".join(f"{w_} {grid[(t, w_)]}" for w_ in sorted(by_weapon) if grid[(t, w_)]))
    print("by rarity and weapon:")
    rarity = Counter()
    for key, n in census.items():
        rarity[(key[4], key[2])] += n
    for (r, w_), n in sorted(rarity.items()):
        print(f"  {n:>5}  {r or '(none)'}, {w_}")
    print("by array and weapon:")
    arr = Counter()
    for key, n in census.items():
        arr[(key[5], key[6], key[2])] += n
    for (a, f, w_), n in sorted(arr.items()):
        print(f"  {n:>5}  {a}{' (' + f + ')' if f else ''}, {w_}")
    says = Counter((r[3], r[7]) for r in uniques)
    print(f"lines on uniques: {len(uniques)}, on {len({(r[0], r[1]) for r in uniques})} named uniques")
    for (w_, o), n in sorted(says.items()):
        print(f"  {n:>5}  {w_}: {o}")
    unsure = [r for r in uniques if r[3] == "weapon" and r[7].startswith("cannot say")]
    print(f"weapon lines on uniques the export cannot say local or global: {len(unsure)}, of which the site "
          f"listed {sum(1 for r in unsure if r[8] == 'local')} as local, "
          f"{sum(1 for r in unsure if r[8] not in ('local', 'not captured'))} otherwise, and "
          f"{sum(1 for r in unsure if r[8] == 'not captured')} not captured")
    print(f"weapon lines on uniques the export says global: "
          f"{sum(1 for r in uniques if r[3] == 'weapon' and r[7] == 'global')}")
    print("lines on uniques, the export's word by the site's (zeros included):")
    for w_ in ("weapon", "not a weapon"):
        for o in ("local", "global", "cannot say"):
            for s in ("local", "global", "not captured"):
                n = sum(1 for r in uniques if r[3] == w_ and r[7].split(":")[0] == o and r[8] == s)
                print(f"  {n:>5}  {w_}: export {o}, site {s}")
    print("what the site says, where a capture holds the unique:")
    for (w_, o, s), n in sorted(Counter((r[3], r[7].split(":")[0], r[8]) for r in uniques).items()):
        print(f"  {n:>5}  {w_}: export {o}, site {s}")
    print(f"wrote {CENSUS_OUT.relative_to(ROOT)}, {UNIQUES_OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
