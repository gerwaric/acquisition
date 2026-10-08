#!/usr/bin/env python3
"""The twin from the export for every line of the ranged family on the copy
(BRIEF.md, step 3): writes ../data/ranged-twins.csv and prints every count the
README states of it.

    ranged-twins.py

scripts/ranged-census.py joins the export to the family's lines on uniques.
This is the same join — its `template`, `numbers`, `mod_rows`, `RANGE`,
`PLACEHOLDER` and `pinned`, imported — for every live `pc` line of the five
unsuffixed `Adds # to # <Type> Damage` on the copy, of any rarity: the mods
whose text holds a row of the line's template with ranges holding the line's
two numbers; of each, the stats the export translates to that text
(`stat_translations.json`) and their `is_local` (`stats.json`). What the
export says of a line is `local`, `global`, `both` (mods found of each),
`no mod found`, or `no stat` (mods found, none carrying a stat the export
translates to the text).

**Which mods are a line's is a reading of the export, and two things in it are
open; each is run both ways, four readings a line.**

How a generation type is chosen for an array:

    named   the types the array, its flags and the rarity name —
              explicitMods on a unique, implicitMods on anything: `unique`
                (ranged-census.py's; a base's implicit is generated `unique`)
              explicitMods off a unique, unflagged or fractured:
                `prefix`, `suffix`, outside the domains `crafted` and
                `brequel_graft`
              explicitMods flagged crafted: the domain `crafted`, any type
              crucibleMods: `crucible_tree`, `crucible_unique_tree`
              enchantMods: `enchantment`; scourgeMods: `scourge_benefit`
    every   every mod of the export, whatever its generation type or domain:
            the most the text join can find, so what it says with one voice
            no choice of types can change

How the item's class is read:

    any     no mod is left out for what the item is (ranged-census.py's)
    admits  a mod is left out where the export says it cannot be on the
            item's base — a mod with a spawn weight above nothing, where the
            first of its spawn weights whose tag the base carries is nothing
            (`base_items.json`, the base by its name, domain `item`; with the
            base's tags, every influence tag of its item class,
            `item_classes.json` — an influence the class can carry is taken
            as carried, the item's own never read); a mod a bench option adds
            (`crafting_bench_options.json`), where no option adding it lists
            the base's item class. A mod the export gives neither (a unique's,
            an implicit's, a crucible node's with every weight nothing) stays:
            the export does not say what it sits on.

The item's side is scripts/ranged-expected.py's class reading from its base
and rarity (`Tables.side`): `weapon`, `not a weapon`, or `open`.

One row a line: `says_named_any` is ranged-census.py's reading on a unique and
is compared with data/ranged-uniques.csv row for row. `differs` reads the four
together against the side: `no` where each says the side's twin (`local` on a
weapon, `global` off one); `the export says the other twin` where one of the
four says the other twin alone; `cannot say` otherwise, with the readings
that fall short.

Inputs, read-only: the owner's store copy (`mode=ro`), the shipped class and
category tables through ranged-expected.py, and the export at e2bd511a. One
absent is named, the file written with its header alone, and the script ends 0.
"""

import csv
import importlib.util
import json
import sqlite3
import subprocess
import sys

sys.dont_write_bytecode = True  # an import here leaves no file behind

from collections import Counter, defaultdict  # noqa: E402
from pathlib import Path  # noqa: E402

TRACK = Path(__file__).resolve().parents[1]
ROOT = TRACK.parent.parent
OUT = TRACK / "data" / "ranged-twins.csv"
UNIQUES = TRACK / "data" / "ranged-uniques.csv"
HEADER = ["side", "class", "base", "rarity", "unique", "array", "flags", "line", "says_named_any",
          "says_named_admits", "says_every_any", "says_every_admits", "differs", "mods_named_any",
          "mods_named_admits", "mods_every_any_count", "mods_every_admits_count", "base_in_export"]
READINGS = ["named/any", "named/admits", "every/any", "every/admits"]
SAYS = ["local", "global", "both", "no mod found", "no stat"]


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


census = load("ranged_census", TRACK / "scripts" / "ranged-census.py")
expected = load("ranged_expected", TRACK / "scripts" / "ranged-expected.py")
POE1, STORE = census.POE1, census.STORE
INPUTS = ("../item-facts/raw/m3/store/mock/GERWARIC_7694.db (read-only), "
          "crates/acquisition-search/reference/classes-v1.toml and categories-v1.toml, "
          "poe1/data/mods.json, stats.json, stat_translations.json, base_items.json, item_classes.json and "
          "crafting_bench_options.json @ e2bd511a; data/ranged-uniques.csv compared, not read into a row")


def named_types(array, flags, rarity):
    """The `named` reading: (generation types or None for any, domains kept or None, domains left out)."""
    if array == "explicitMods" and "crafted" in flags:
        return None, {"crafted"}, set()
    if array == "explicitMods" and rarity != "Unique":
        return {"prefix", "suffix"}, None, {"crafted", "brequel_graft"}
    if array in ("explicitMods", "implicitMods"):
        return {"unique"}, None, set()
    if array == "crucibleMods":
        return {"crucible_tree", "crucible_unique_tree"}, None, set()
    if array == "enchantMods":
        return {"enchantment"}, None, set()
    if array == "scourgeMods":
        return {"scourge_benefit"}, None, set()
    return set(), None, set()


def write(rows):
    with OUT.open("w", newline="") as fh:
        fh.write(f"# generated by scripts/ranged-twins.py from {INPUTS}\n")
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(HEADER)
        w.writerows(rows)


def main():
    if sys.argv[1:]:
        print(__doc__)
        return 2
    absent = [str(p.relative_to(ROOT) if p.is_relative_to(ROOT) else p)
              for p in (STORE, POE1 / "data" / "mods.json") if not p.exists()]
    if not absent:
        head = subprocess.run(["git", "-C", str(POE1), "rev-parse", "HEAD"], capture_output=True, text=True)
        if head.stdout.strip() != census.POE1_COMMIT:
            absent.append(f"{POE1} at {census.POE1_COMMIT} (it is at {head.stdout.strip() or 'no commit'})")
    if absent:
        write([])
        print("ABSENT: " + "; ".join(absent) + " — no line is joined; every count of it is an open question")
        print(f"wrote {OUT.relative_to(ROOT)}: 0 rows")
        return 0

    data = POE1 / "data"
    stats = json.loads((data / "stats.json").read_text())
    renders = defaultdict(set)  # template -> the stats the export translates to it
    for entry in json.loads((data / "stat_translations.json").read_text()):
        for s in entry.get("English", []):
            t = census.PLACEHOLDER.sub("#", s["string"]).replace("+#", "#")
            if t in census.FAMILY:
                renders[t].update(entry["ids"])
    bench = defaultdict(set)  # mod id -> the item classes of every bench option adding it
    for option in json.loads((data / "crafting_bench_options.json").read_text()):
        mod_id = option["actions"].get("add_explicit_mod")
        if mod_id:
            bench[mod_id].update(option["item_classes"])
    influence = {name: set(c.get("influence_tags") or [])
                 for name, c in json.loads((data / "item_classes.json").read_text()).items()}
    bases = defaultdict(list)  # name -> [(item class, tags)]
    for base in json.loads((data / "base_items.json").read_text()).values():
        if base["domain"] == "item":
            bases[base["name"]].append((base["item_class"],
                                        set(base["tags"]) | influence.get(base["item_class"], set())))

    by_template = defaultdict(list)  # template -> [mod]
    family_mods = Counter()
    for mod_id, mod in json.loads((data / "mods.json").read_text()).items():
        for t, spans in census.mod_rows(mod.get("text") or ""):
            if t not in census.FAMILY:
                continue
            behind = [s["id"] for s in mod["stats"] if s["id"] in renders[t]]
            by_template[t].append({
                "id": mod_id, "spans": spans, "domain": mod["domain"], "type": mod["generation_type"],
                "local": {bool(stats.get(s, {}).get("is_local")) for s in behind},
                "weights": [(w["tag"], w["weight"]) for w in mod["spawn_weights"]],
            })
            family_mods[(mod["domain"], mod["generation_type"])] += 1

    def admitted(mod, base):
        """False where the export says the mod cannot be on this base; True where it can or does not say."""
        entries = bases.get(base, [])
        if mod["id"] in bench:
            return not entries or any(cls in bench[mod["id"]] for cls, _ in entries)
        if any(weight > 0 for _, weight in mod["weights"]):
            if not entries:
                return True
            for _, tags in entries:
                first = next((weight for tag, weight in mod["weights"] if tag in tags), 0)
                if first > 0:
                    return True
            return False
        return True

    def says(found):
        if not found:
            return "no mod found"
        voices = set().union(*(m["local"] for m in found))
        return ("local" if voices == {True} else "global" if voices == {False} else "both" if voices
                else "no stat")

    tables = expected.Tables()
    db = sqlite3.connect(f"file:{STORE}?mode=ro", uri=True)
    rows, keys = [], []
    for item_id, rarity, body in db.execute(
            "select id, rarity, json from items where removed_at is null and realm = 'pc' order by id"):
        body = json.loads(body)
        base = body.get("baseType") or body.get("typeLine") or ""
        rarity = rarity or ""
        side = tables.side({"base": base, "rarity": rarity})[0] or "open"
        cls = "|".join(sorted(tables.classes.get(base, set()))) or "not in the class table"
        for array, entries in body.items():
            if not (array.endswith("Mods") and isinstance(entries, list)):
                continue
            for entry in entries:
                text = entry.get("description") if isinstance(entry, dict) else entry
                if not isinstance(text, str):
                    continue
                flags = expected.flags_shown(entry.get("flags")) if isinstance(entry, dict) else ""
                for row in text.split("\n"):
                    t = census.template(row)
                    if t not in census.FAMILY:
                        continue
                    lo, hi = census.numbers(row)[:2]
                    every = [m for m in by_template[t] if len(m["spans"]) >= 2
                             and m["spans"][0][0] <= lo <= m["spans"][0][1]
                             and m["spans"][1][0] <= hi <= m["spans"][1][1]]
                    types, only, never = named_types(array, flags.split(";"), rarity)
                    named = [m for m in every if (types is None or m["type"] in types)
                             and (only is None or m["domain"] in only) and m["domain"] not in never]
                    found = [named, [m for m in named if admitted(m, base)],
                             every, [m for m in every if admitted(m, base)]]
                    said = [says(f) for f in found]
                    want = "local" if side == "weapon" else "global" if side == "not a weapon" else ""
                    other = {"local": "global", "global": "local"}.get(want)
                    if all(s == want for s in said):
                        differs = "no"
                    elif other in said:
                        differs = "the export says the other twin: " + ", ".join(
                            r for r, s in zip(READINGS, said) if s == other)
                    else:
                        differs = "cannot say: " + ", ".join(
                            f"{r} {s}" for r, s in zip(READINGS, said) if s != want)
                    ids = [";".join(sorted({m["id"] for m in f})) for f in found]
                    name = body.get("name", "") if rarity == "Unique" else ""
                    rows.append([side, cls, base, rarity, name, array, flags, row] + said + [
                        differs, ids[0], ids[1], len({m["id"] for m in found[2]}),
                        len({m["id"] for m in found[3]}), "yes" if base in bases else "no"])
                    keys.append(item_id)
    order = sorted(range(len(rows)), key=lambda i: [str(c) for c in rows[i]])
    items = len(set(keys))
    rows = [rows[i] for i in order]
    write(rows)

    print("the family's mods in the export, by domain and generation type: " + "; ".join(
        f"{d}/{g} {n}" for (d, g), n in sorted(family_mods.items())))
    print(f"{len(rows)} live pc lines of the five templates on {items} items; by side: " + ", ".join(
        f"{s} {n}" for s, n in sorted(Counter(r[0] for r in rows).items())))
    print(f"bases of those lines the export's base_items.json does not hold by name: "
          f"{len({r[2] for r in rows if r[17] == 'no'})} ({sum(1 for r in rows if r[17] == 'no')} lines); "
          f"held under several entries: {len({r[2] for r in rows if len(bases.get(r[2], [])) > 1})}")
    for index, reading in enumerate(READINGS):
        print(f"reading {reading} — side / array / flags: " + " / ".join(SAYS) + ":")
        grid = defaultdict(Counter)
        for r in rows:
            grid[(r[0], r[5], r[6] or "(none)")][r[8 + index]] += 1
        for key, got in sorted(grid.items()):
            print(f"  {' / '.join(key)}: " + " / ".join(str(got[s]) for s in SAYS))
        totals = Counter((r[0], r[8 + index]) for r in rows)
        for side in sorted({r[0] for r in rows}):
            print(f"  {side}, all: " + " / ".join(str(totals[(side, s)]) for s in SAYS))
    print("the four readings a line, where they do not say one thing — side / array / flags / "
          + " | ".join(READINGS) + ": lines:")
    for key, n in sorted(Counter((r[0], r[5], r[6] or "(none)", " | ".join(r[8:12])) for r in rows
                                 if len(set(r[8:12])) > 1).items()):
        print(f"  {' / '.join(key)}: {n}")
    print(f"lines the four readings say one thing of: {sum(1 for r in rows if len(set(r[8:12])) == 1)}")
    print("the export against the item's side — side / rarity / array / flags: lines:")
    for kind in ("no", "the export says the other twin", "cannot say"):
        held = [r for r in rows if r[12].split(":")[0] == kind]
        print(f"  {kind}: {len(held)}")
        for key, n in sorted(Counter((r[0], r[3], r[5], r[6] or "(none)", r[12]) for r in held).items()):
            if kind != "no":
                print(f"    {' / '.join(key[:4])} — {key[4]}: {n}")
    for r in rows:
        if r[12].startswith("the export says the other twin"):
            print(f"  DIFFERS  {r[0]} · {r[3]} {r[4] or r[2]} ({r[1]}) · {r[5]}[{r[6]}] · {r[7]!r} · "
                  f"{' | '.join(r[8:12])} · named/any mods: {r[13] or 'none'}")

    # The unique rows beside ranged-census.py's.
    if not UNIQUES.exists():
        print(f"ABSENT: {UNIQUES.relative_to(ROOT)} — the unique rows are compared with nothing")
    else:
        with UNIQUES.open(newline="\n") as fh:
            next(fh)
            theirs = Counter()
            for r in csv.DictReader(fh):
                if r["realm"] != "pc":
                    continue
                word = r["export_says"]
                word = ("both" if word.endswith("say local and global") else "no stat" if "carry no stat" in word
                        else "no mod found" if word.startswith("cannot say") else word)
                theirs[(r["unique"], r["base"], r["array"], r["line"], word, r["mods"])] += 1
        ours = Counter((r[4], r[2], r[5], r[7], r[8], r[13]) for r in rows if r[3] == "Unique")
        same = sum((ours & theirs).values())
        print(f"the unique rows: {sum(ours.values())} here, {sum(theirs.values())} pc rows in "
              f"data/ranged-uniques.csv; the same by unique, base, array, line, what the export says "
              f"(named/any) and the mods' ids: {same}; here alone {sum((ours - theirs).values())}, there alone "
              f"{sum((theirs - ours).values())} — the unique rows are "
              f"{'' if ours == theirs else 'NOT '}data/ranged-uniques.csv's")
    print(f"wrote {OUT.relative_to(ROOT)}: {len(rows)} rows")
    return 0


if __name__ == "__main__":
    sys.exit(main())
