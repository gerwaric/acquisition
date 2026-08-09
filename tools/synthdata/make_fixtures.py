#!/usr/bin/env python3
"""Extract the checked-in RePoE fixture subset under tools/synthdata/fixtures/.

The fixtures are a closed, deterministic slice of real RePoE data — a few
bases per item class the generator can emit, the spawn-legal mods (with
display text) for exactly those bases, and the stat translations those mods
reference — small enough to commit, so `generate.py --repoe-dir fixtures`
runs offline in CI with no network and no live-data drift.

Selection is fully deterministic (sorted keys, fixed counts) and the output
is canonical JSON, so re-running this script against the same upstream data
is a no-op diff; the freshness workflow uses that property to tell "upstream
version bumped, subset unchanged" from "the shapes this repo relies on moved".

Usage: make_fixtures.py [--out fixtures/] [--from-dir .cache/<version>/]
"""

import argparse
import json
import pathlib

import generate
import repoe_data

# Every class the generator can draw a base from (tab makers, character
# slots, coverage/quirk rares), so no pool is empty under fixtures.
CLASSES = (
    generate.EQUIP_CLASSES | generate.FLASK_CLASSES | generate.JEWEL_CLASSES
    | generate.STACKABLE_CLASSES | {generate.MAP_CLASS}
    | {"Active Skill Gem", "Support Skill Gem", "MapFragment", "Breachstone",
       "MapKey", "DelveSocketableCurrency", "DelveStackableSocketableCurrency"}
)
BASES_PER_CLASS = 2
MODS_PER_KIND = 6  # prefixes and suffixes kept per base

# generate.py's dedicated-stash pools select StackableCurrency by name; keep
# one base per marker so Essence/Blight/Delirium tabs stay populated.
CURRENCY_MARKERS = ["Essence", "Remnant of", "Oil", "Delirium Orb", "Simulacrum"]


def pick_bases(base_items):
    released = {k: v for k, v in sorted(base_items.items())
                if v.get("release_state") == "released" and v.get("name")}
    chosen = {}
    by_class = {}
    for k, v in released.items():
        by_class.setdefault(v["item_class"], []).append((k, v))
    for cls in sorted(CLASSES):
        cands = sorted(by_class.get(cls, []),
                       key=lambda kv: (kv[1].get("drop_level", 1), kv[0]))
        for k, v in cands[:BASES_PER_CLASS]:
            chosen[k] = v
    for marker in CURRENCY_MARKERS:
        for k, v in by_class.get("StackableCurrency", []):
            if marker in v["name"]:
                chosen[k] = v
                break
    return chosen


def pick_mods(mods, bases):
    """Implicits of the chosen bases plus, per base, the first few spawn-legal
    prefixes and suffixes that carry display text (the same legality test as
    RepoeIndex.eligible_mods, at the generator's max ilvl)."""
    chosen = {}
    for base in bases.values():
        for mid in base.get("implicits") or []:
            if mid in mods:
                chosen[mid] = mods[mid]
        tags = set(base.get("tags") or [])
        kept = {"prefix": 0, "suffix": 0}
        for mid, m in sorted(mods.items()):
            gen = m.get("generation_type")
            if gen not in kept or kept[gen] >= MODS_PER_KIND:
                continue
            if m.get("domain") != base.get("domain") or m.get("is_essence_only") \
                    or not m.get("spawn_weights") or not m.get("text") \
                    or m.get("required_level", 1) > 86:
                continue
            for sw in m["spawn_weights"]:
                if sw["tag"] in tags:
                    if sw["weight"] > 0:
                        chosen[mid] = m
                        kept[gen] += 1
                    break
    return chosen


def main():
    here = pathlib.Path(__file__).resolve().parent
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(here / "fixtures"))
    ap.add_argument("--from-dir", help="read RePoE files from this directory"
                    " instead of the live cache/web")
    args = ap.parse_args()

    data = repoe_data.load(args.from_dir)
    bases = pick_bases(data["base_items"])
    mods = pick_mods(data["mods"], bases)
    classes = {k: v for k, v in sorted(data["item_classes"].items())
               if k in CLASSES}
    stat_ids = {s["id"] for m in mods.values() for s in m.get("stats") or []}
    translations = [t for t in data["stat_translations"]
                    if stat_ids & set(t.get("ids") or [])]

    out = pathlib.Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    for name, payload in [("base_items.min.json", bases),
                          ("item_classes.min.json", classes),
                          ("mods.min.json", mods),
                          ("stat_translations.min.json", translations)]:
        (out / name).write_text(json.dumps(
            payload, sort_keys=True, separators=(",", ":")) + "\n")
    # The generator reports this as its RePoE version, marking fixture-lane
    # output; the real upstream version lives in source.json for the
    # freshness check.
    (out / "version.txt").write_text("fixture-v1\n")
    (out / "source.json").write_text(json.dumps({
        "source_version": data["version"],
        "bases": len(bases), "mods": len(mods),
        "stat_translations": len(translations),
    }, indent=2, sort_keys=True) + "\n")
    print(f"wrote {out}: {len(bases)} bases, {len(mods)} mods,"
          f" {len(translations)} translations"
          f" (source {data['version']})")


if __name__ == "__main__":
    main()
