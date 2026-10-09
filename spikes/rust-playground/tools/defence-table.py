#!/usr/bin/env python3
"""The defence table (C101, C106, C68; item search step 9d-A): what the base
defence percentile, `pseudo.defence_pct`, reads beside the item — each base's
defence ranges, and each text a local defence stat displays with what it does
— regenerated from the owner's pinned RePoE clone (SURFACES.md, the RePoE
row; search/repoe/MANIFEST.md) and one committed extract into the reviewed
file the search crate ships.

    python3 tools/defence-table.py            # writes crates/acquisition-search/reference/defences-v1.toml
    python3 tools/defence-table.py --check    # exits 1 when the file on disk differs from what the sources give

Inputs (read-only; nothing is fetched; the clone must sit at POE1_COMMIT or
this refuses to run — a pull is a new commit here and a reviewed diff of the
table, never a silent regeneration):
  ../poe1/data/base_items.json            every base: its name and `properties.armour`, `.evasion`,
                                          `.energy_shield`, `.ward` as `{min, max}` or null, `.block`
  search/repoe/data/trade-stat-map.csv    the trade site's stat texts joined to the export's stats
                                          (search/repoe/scripts/trade-stat-map.py, from the same clone's
                                          stat_translations.json and the site's stats of 2026-09-12)
  search/repoe/data/base-defences.csv     the track's extract of the same ranges: read only to show that
                                          the ranges written here are that file's, row for row

Rules, each a reading of a source and never a judgment (C106). The rulings the
field rests on are `search/LEDGER.md`, "Holes ruled", the 9d rows (J5, J6, J8,
J33 to J35); what reads this table is `src/pseudo.rs`.
  1. The ranges. A base enters as base-defences.csv holds it: every entry of
     base_items.json carrying a defence range or a block value, in the order of
     its name and then its id, with its name and the minimum and maximum of each
     defence it ranges — and nothing else of the export: no id, no class, no
     release state, no block. A row is an entry, not a name: a name with several
     entries (Two-Toned Boots) keeps a row for each, and the field tells them
     apart by the defences the item displays. An entry that ranges no defence —
     it carries block alone — is a row with no range. A range whose minimum is
     its maximum is written as it is, and the field says of it what J33 rules.
  2. The defence lines. A text enters where the trade site displays it for one
     of the export's local defence stats (STATS, below: each stat with what it
     does, as both community tools state it — Awakened PoE Trade
     `calc-q20.ts` L5-L38, Path of Building `Item.lua` L2347-L2365): the site's
     text without its `(Local)` mark, the sign carried in the number (`+#` is
     `#`, C90). The `reduced` spelling of an `increased` text enters beside it:
     it is the same stat below nothing.
  3. A twin text. A text of rule 2 that the site also displays, with no
     `(Local)` mark, for a stat that is none of STATS, is a twin: a private
     item shows the local stat and the global one alike, and which it is the
     item's mark tells (J8). The `reduced` spelling of a twin is a twin.
  4. `Quality does not increase Defences` enters as the text that drops the
     quality term (J6).
Nothing is renamed, merged or dropped beyond these; the counts the header
prints are measured at generation, and the script stops where they are not
the track's: 476 rows, 474 names, 22 texts, 10 of them twin
(search/pseudo-stats/scripts/percentile-expected.py prints the same).
"""

import collections
import csv
import json
import os
import re
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
# the clones sit beside the repository (search/repoe/MANIFEST.md, "The clones")
POE1 = os.path.normpath(os.path.join(ROOT, "..", "..", "..", "poe1"))
POE1_COMMIT = "e2bd511a0133bbe6c1ab548ef1285cb99f3cf0e9"  # search/repoe/MANIFEST.md
TRADE_MAP = os.path.join(ROOT, "search", "repoe", "data", "trade-stat-map.csv")
EXTRACT = os.path.join(ROOT, "search", "repoe", "data", "base-defences.csv")
OUT = os.path.join(ROOT, "crates", "acquisition-search", "reference", "defences-v1.toml")
VERSION = 1
REALMS = ["pc", "xbox", "sony"]  # the PoE1 export, as the class table's; poe2 has no table yet

# The four defences, as the export keys them, in the order the game displays them.
DEFENCES = ["armour", "evasion", "energy_shield", "ward"]

# Rule 2: the export's local defence stats, and what each does to which
# defence — `add` before the increase, or `increase`.
STATS = {
    "local_base_physical_damage_reduction_rating": ("add", ["armour"]),
    "local_base_evasion_rating": ("add", ["evasion"]),
    "local_energy_shield": ("add", ["energy_shield"]),
    "local_ward": ("add", ["ward"]),
    "local_evasion_rating_and_energy_shield": ("add", ["evasion", "energy_shield"]),
    "local_physical_damage_reduction_rating_+%": ("increase", ["armour"]),
    "local_evasion_rating_+%": ("increase", ["evasion"]),
    "local_energy_shield_+%": ("increase", ["energy_shield"]),
    "local_ward_+%": ("increase", ["ward"]),
    "local_armour_and_evasion_+%": ("increase", ["armour", "evasion"]),
    "local_armour_and_energy_shield_+%": ("increase", ["armour", "energy_shield"]),
    "local_evasion_and_energy_shield_+%": ("increase", ["evasion", "energy_shield"]),
    "local_armour_and_evasion_and_energy_shield_+%": ("increase", ["armour", "evasion", "energy_shield"]),
}
# Rule 4.
NO_QUALITY = "local_quality_does_not_increase_defences"

# What the track counted, which this script must count too.
EXPECTED = {"rows": 476, "names": 474, "texts": 22, "twins": 10}


def pinned(path, commit):
    head = subprocess.run(["git", "-C", path, "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    if head != commit:
        sys.exit(f"{path} is at {head}, not the pinned {commit}: a pull is a new commit in this script and a reviewed diff of the table")


def toml_str(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def ranges():
    """Rule 1: [(name, {defence: (min, max)})], an entry a row, by name then id."""
    bases = json.load(open(os.path.join(POE1, "data", "base_items.json")))
    rows = []
    for key, base in sorted(bases.items(), key=lambda kv: (kv[1]["name"], kv[0])):
        properties = base.get("properties") or {}
        ranged = {d: properties.get(d) for d in DEFENCES}
        if not any(r is not None for r in ranged.values()) and properties.get("block") is None:
            continue
        rows.append((base["name"], {d: (r["min"], r["max"]) for d, r in ranged.items() if r is not None}))
    return rows


def extract():
    """base-defences.csv's rows in the shape `ranges` gives: the track's extract."""
    with open(EXTRACT, newline="") as fh:
        next(fh)  # the line naming the script that wrote it
        return [(row["name"], {d: (int(row[f"{d}_min"]), int(row[f"{d}_max"])) for d in DEFENCES if row[f"{d}_min"]})
                for row in csv.DictReader(fh)]


def shown(text):
    """A trade text as a private item displays it: no `(Local)` mark, the sign in the number."""
    return re.sub(r" \(Local\)$", "", text).replace("+#", "#")


def lines():
    """Rules 2 to 4: [(text, what it does, the defences it does it to, whether a twin)], by text."""
    with open(TRADE_MAP, newline="") as fh:
        next(fh)  # the line naming the script that wrote it
        rows = list(csv.DictReader(fh))
    does = {}
    for row in rows:
        stat = row["repoe_ids"]
        if stat == NO_QUALITY:
            does.setdefault(shown(row["trade_text"]), ("drop_quality", []))
        elif stat in STATS:
            what, to = STATS[stat]
            text = shown(row["trade_text"])
            does.setdefault(text, (what, to))
            if what == "increase":
                assert "increased" in text, text
                does.setdefault(text.replace("increased", "reduced"), ("reduce", to))
    # rule 3: a text of a local stat that a stat none of STATS displays too, unmarked
    elsewhere = {shown(row["trade_text"]) for row in rows
                 if row["repoe_ids"] not in STATS and not row["trade_text"].endswith(" (Local)")}
    twins = {text for text, (what, _) in does.items() if what in ("add", "increase") and text in elsewhere}
    twins |= {text.replace("increased", "reduced") for text in twins if "increased" in text}
    return [(text, what, to, text in twins) for text, (what, to) in sorted(does.items())]


def main():
    check = "--check" in sys.argv[1:]
    pinned(POE1, POE1_COMMIT)
    game = open(os.path.join(POE1, "version.txt")).read().strip()
    bases, texts = ranges(), lines()

    # the ranges are the track's extract, row for row
    theirs = extract()
    if bases != theirs:
        differing = sum(1 for a, b in zip(bases, theirs) if a != b)
        sys.exit(f"the ranges are not search/repoe/data/base-defences.csv's: {len(bases)} rows here, {len(theirs)} "
                 f"there, {differing} of the rows both hold differ")
    by_name = collections.Counter(name for name, _ in bases)
    several = sorted(name for name, n in by_name.items() if n > 1)
    flat = sorted({name for name, ranged in bases for low, high in ranged.values() if low == high})
    none = sorted(name for name, ranged in bases if not ranged)
    twins = [text for text, _, _, twin in texts if twin]
    counted = {"rows": len(bases), "names": len(by_name), "texts": len(texts), "twins": len(twins)}
    if counted != EXPECTED:
        sys.exit(f"the table counts {counted} where the track counted {EXPECTED}: a pull or a new extract is a "
                 f"reviewed diff, the counts in this script among it")

    out = [
        "# The defence table — v1 (C101, C106, C68; decisions/search.md), item search step 9d-A.",
        "#",
        "# Reference data: reviewed, committed, shipped inside the binary",
        "# (`include_str!` in `src/defences.rs`), read-only, never in a store file,",
        "# printed with the definition `acq search --describe defence_pct` gives,",
        "# cited by version in every basis (C98). Generated by tools/defence-table.py",
        "# from the owner's pinned RePoE clone and the committed extract",
        "# search/repoe/data/trade-stat-map.csv, and reviewed as a diff at each pull",
        "# (the admission test, search/DESIGN.md, C106): data, what the game files",
        "# state — never a convention and never a judgment. Its rules are the",
        "# script's docstring.",
        "#",
        "# What a `base` row means: an item of the base named rolled each defence the",
        "# row ranges between its minimum and its maximum, both included. A row is an",
        "# entry of the export, never a name: a name on several rows is several bases,",
        "# told apart by the defences an item displays. A row with no range is a base",
        "# that ranges no defence. Nothing else of the export is here.",
        "#",
        "# What a `line` row means: a line displaying the text is, where it is the",
        "# item's own, the local stat that does what the row says to the defences it",
        "# names — `add` to the roll before any increase, `increase` the sum, `reduce`",
        "# the same stat below nothing, `drop_quality` the enchant under which quality",
        "# does not multiply. `twin` marks a text a global stat displays too: which of",
        "# the two a line is, the text does not say (J8).",
        "#",
        f"# Measured at generation: {len(bases)} base rows over {len(by_name)} names — the rows of",
        "# search/repoe/data/base-defences.csv, each one — of which held more than",
        f"# once: {', '.join(f'{n} ({by_name[n]})' for n in several)}; a minimum that is its maximum:",
        f"# {', '.join(flat)}; ranging no defence: {', '.join(none)}.",
        f"# {len(texts)} line texts, {len(twins)} of them twin.",
        "",
        f"version = {VERSION}",
        f"realms = [{', '.join(toml_str(r) for r in REALMS)}]",
        f"source = {toml_str(f'RePoE poe1 export @ {POE1_COMMIT[:8]} (game {game}): base_items.json for the ranges; for the lines, the trade site’s stat texts of 2026-09-12 joined to the export’s stats (search/repoe/data/trade-stat-map.csv); the owner’s clone of 2026-09-13, SURFACES.md')}",
        f"game = {toml_str(game)}",
        'generated_by = "tools/defence-table.py"',
        "",
    ]
    for name, ranged in bases:
        out.append("[[base]]")
        out.append(f"name = {toml_str(name)}")
        for defence in DEFENCES:
            if defence in ranged:
                out.append(f"{defence} = [{ranged[defence][0]}, {ranged[defence][1]}]")
        out.append("")
    for text, what, to, twin in texts:
        out.append("[[line]]")
        out.append(f"text = {toml_str(text)}")
        out.append(f"does = {toml_str(what)}")
        out.append(f"to = [{', '.join(toml_str(d) for d in to)}]")
        out.append(f"twin = {'true' if twin else 'false'}")
        out.append("")
    text = "\n".join(out)

    summary = (f"{len(bases)} base rows, {len(by_name)} names, the same as search/repoe/data/base-defences.csv row "
               f"for row; held more than once: {', '.join(f'{n} ({by_name[n]})' for n in several)}; a minimum that "
               f"is its maximum: {', '.join(flat)}; ranging no defence: {', '.join(none)}; {len(texts)} line texts, "
               f"{len(twins)} of them twin: {'; '.join(twins)}")
    if check:
        if not os.path.exists(OUT) or open(OUT, newline="").read() != text:
            sys.exit(f"{OUT} differs from what the clone at {POE1_COMMIT[:8]} and the extract give: regenerate and review the diff")
        print(f"ok      {os.path.relpath(OUT, ROOT)} is what the sources give: {summary}")
        return
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", newline="\n") as f:
        f.write(text)
    print(f"{os.path.relpath(OUT, ROOT)}: {summary}")
    for line, what, to, twin in texts:
        print(f"  {line}: {what} {'/'.join(to) or '-'}{' (twin)' if twin else ''}")


if __name__ == "__main__":
    main()
