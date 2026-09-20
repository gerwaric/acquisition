#!/usr/bin/env python3
"""M2 (search/BUILD-PLAN.md): the Rust deriver against the census, over one input.

The committed census (data/mod-templates.csv) was taken over three inputs, so
this re-runs the census's own template rule (census.py's `template`) over one —
the live items of raw/spike-GERWARIC_7694-2026-09-13.db — hands the same rows to
the deriver (crates/acquisition-search/examples/derive-census.rs), and sets the
two tables against each other: templates, items, lines, flags, numbers, and
the ranged split (ranged-split.py's `classify`).

The deriver departs from the census by four named normalisations, two
exclusions and one addition (a vaal gem's `hybrid.explicitMods`, which the
census never read; here it is the array `hybridMods`). Each is applied here, one at a time, to the census's input, and
counted; the deriver's table must then equal the normalised census exactly.
What is left over is unexplained, and the script exits 1.

Writes raw/m2/ (gitignored). Run, from anywhere, after `cargo build`:
  python3 search/item-facts/scripts/m2-differential.py
"""
import collections
import importlib.util
import json
import os
import re
import sqlite3
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TRACK = os.path.dirname(HERE)
ROOT = os.path.abspath(os.path.join(TRACK, "..", ".."))
SPIKE = os.path.join(TRACK, "raw", "spike-GERWARIC_7694-2026-09-13.db")
OUT = os.path.join(TRACK, "raw", "m2")


def load(name):
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), os.path.join(HERE, name + ".py"))
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


census = load("census")
ranged_split = load("ranged-split")

# ---- the named departures, each a function of the displayed text
STYLE = re.compile(r"<[^<>]*>\{([^{}]*)\}")
BRACKET = re.compile(r"\[([^\]]*)\]")
THOUSANDS = re.compile(r"(?<![\d#])[-+]?\d+(?:,\d{3}(?!\d))*(?:\.\d+)?")


def rows_lf(text):
    return text.replace("\r\n", "\n").replace("\r", "\n")


def unstyled(text):
    while True:
        text, n = STYLE.subn(r"\1", text)
        if not n:
            return text


def unbracketed(text):
    return BRACKET.sub(lambda m: m.group(1).rsplit("|", 1)[-1], text)


STEPS = [("row break `\\r\\n` read as `\\n`", rows_lf), ("`<style>{…}` reduced", unstyled), ("`[Tag|Display]` reduced", unbracketed)]


def tally():
    return {"items": 0, "lines": 0, "numbers": 0, "flags": collections.Counter()}


def main():
    os.makedirs(OUT, exist_ok=True)
    db = sqlite3.connect(f"file:{SPIKE}?mode=ro", uri=True)
    q = """SELECT id, realm, league, location_kind, location_id, container, socketed_in, first_seen, last_seen, removed_at, json
           FROM items WHERE removed_at IS NULL ORDER BY id"""
    cols = ["id", "realm", "league", "location_kind", "location_id", "container", "socketed_in", "first_seen", "last_seen", "removed_at"]

    as_census = collections.defaultdict(tally)   # the census's rule, untouched
    expected = collections.defaultdict(tally)    # after the named departures
    moved = collections.Counter()                # departure -> lines whose template it changed
    moved_templates = collections.defaultdict(set)
    excluded = collections.Counter()
    items = 0
    jsonl = os.path.join(OUT, "items.jsonl")
    with open(jsonl, "w") as out:
        for row in db.execute(q):
            items += 1
            out.write(json.dumps({"facts": dict(zip(cols, row[:-1])), "body": row[-1]}) + "\n")
            body = json.loads(row[-1])
            seen_c, seen_e = set(), set()
            arrays = [(k, v, True) for k, v in body.items() if k.endswith("Mods") and isinstance(v, list)]
            hybrid = (body.get("hybrid") or {}).get("explicitMods")
            if isinstance(hybrid, list):
                arrays.append(("hybridMods", hybrid, False))
            for k, v, in_census in arrays:
                for line in v:
                    if isinstance(line, dict):
                        text, flags = line.get("description", line.get("type", "")), line.get("flags") or {}
                    else:
                        text, flags = str(line), {}
                    tpl = census.template(text).replace("\\n", "\n")
                    if in_census:
                        c = as_census[(k, tpl)]
                        c["lines"] += 1
                        if (k, tpl) not in seen_c:
                            seen_c.add((k, tpl))
                            c["items"] += 1
                    else:
                        excluded["`hybrid.explicitMods`, a vaal gem's base skill: ADDED as the source `hybrid`"] += 1
                    if k == "ultimatumMods":
                        excluded["`ultimatumMods`: ids of what `explicitMods` displays, not lines: EXCLUDED"] += 1
                        continue
                    if text == "":
                        excluded["an empty line (the spacer rows of an essence's description) displays nothing: EXCLUDED"] += 1
                        continue
                    for label, step in STEPS:
                        after = step(text)
                        if census.NUM.sub("#", after) != census.NUM.sub("#", text):
                            moved[label] += 1
                            moved_templates[label].add(tpl)
                        text = after
                    final = THOUSANDS.sub("#", text)
                    if final != census.NUM.sub("#", text):
                        moved["`1,500` read as one number"] += 1
                        moved_templates["`1,500` read as one number"].add(tpl)
                    e = expected[(k, final)]
                    e["lines"] += 1
                    e["numbers"] += 0 if k == "veiledMods" else len(THOUSANDS.findall(text))
                    for fk, fv in flags.items():
                        if fv is True:
                            e["flags"][fk] += 1
                    if (k, final) not in seen_e:
                        seen_e.add((k, final))
                        e["items"] += 1

    rust_json = os.path.join(OUT, "rust.json")
    exe = os.path.join(ROOT, "target", "debug", "examples", "derive-census")
    subprocess.run(["cargo", "build", "-q", "-p", "acquisition-search", "--example", "derive-census"], cwd=ROOT, check=True)
    with open(jsonl) as src, open(rust_json, "w") as dst:
        subprocess.run([exe], stdin=src, stdout=dst, check=True)
    rust = json.load(open(rust_json))
    got = {(r["source"] + "Mods", r["template"]): r for r in rust["rows"]}

    # ---- the differential
    unexplained = []
    for key in sorted(set(expected) | set(got)):
        e, g = expected.get(key), got.get(key)
        if e is None or g is None:
            unexplained.append((key, "only in " + ("the deriver" if e is None else "the census")))
            continue
        for field in ("items", "lines", "numbers"):
            if e[field] != g[field]:
                unexplained.append((key, f"{field}: census {e[field]}, deriver {g[field]}"))
        if dict(e["flags"]) != g["flags"]:
            unexplained.append((key, f"flags: census {dict(e['flags'])}, deriver {g['flags']}"))
        want = ranged_split.classify(key[1]) in ("A", "B")
        if want != g["ranged"]:
            unexplained.append((key, f"ranged: census {want}, deriver {g['ranged']}"))

    def summary(table):
        by = collections.defaultdict(lambda: [0, 0])
        for (k, _), t in table.items():
            by[k][0] += 1
            by[k][1] += t["lines"]
        return by

    print(f"input: {os.path.basename(SPIKE)}, live items {items:,}; the deriver saw {rust['items']:,}")
    print(f"deriver: {rust['properties']:,} properties, {rust['displayed']:,} displayed rows, {rust['derive_seconds']:.2f} s deriving (debug build)")
    print("\n| array | census templates / lines | deriver templates / lines |\n| --- | ---: | ---: |")
    c_by, e_by = summary(as_census), summary({k: {"lines": g["lines"]} for k, g in got.items()})
    for k in sorted(set(c_by) | set(e_by), key=lambda k: -max(c_by[k][1], e_by[k][1])):
        print(f"| `{k}` | {c_by[k][0]:,} / {c_by[k][1]:,} | {e_by[k][0]:,} / {e_by[k][1]:,} |")
    print(f"| all | {len(as_census):,} / {sum(t['lines'] for t in as_census.values()):,} | {len(got):,} / {sum(g['lines'] for g in got.values()):,} |")
    print("\ndepartures (lines whose template moved / census templates touched):")
    for label, n in excluded.items():
        print(f"- {label}: {n:,} lines")
    for label, n in moved.items():
        print(f"- {label}: {n:,} / {len(moved_templates[label]):,}")
    split = collections.Counter(ranged_split.classify(t) for (_, t) in got)
    lines_by = collections.Counter()
    for (_, t), g in got.items():
        lines_by[ranged_split.classify(t)] += g["lines"]
    print("\nranged split over the deriver's templates (rows of (array, template); ranged-split.py's cases):")
    for case, label in ranged_split.CASES.items():
        print(f"- {case}: {split[case]:,} rows, {lines_by[case]:,} lines — {label}")
    print(f"- ranged by the deriver: {sum(1 for g in got.values() if g['ranged']):,} rows")
    print("\nunread, by part (items):")
    for u in rust["unread"] or [{"part": "none", "problem": "", "items": 0}]:
        print(f"- {u['part']} {u['problem']}: {u['items']:,}")
    print(f"\nunexplained differences: {len(unexplained)}")
    for key, why in unexplained[:40]:
        print(f"  {key!r}: {why}")
    return 1 if unexplained else 0


if __name__ == "__main__":
    sys.exit(main())
