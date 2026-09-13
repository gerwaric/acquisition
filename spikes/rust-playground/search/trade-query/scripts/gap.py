#!/usr/bin/env python3
"""The gap: what each trade filter would read from a private-API item, or that
it reads the market. Writes data/gap.csv from data/grammar.json.

The assignment is a rule table, not an inference. `local:property N` comes
from the site's own property-type table (grammar.json,
property_type_to_field): the site renders that property under the filter's
field, so the filter reads that property. `local:field <name>` names an item
JSON field the site's renderer reads (grammar.json, and the popup's field
list in README "Findings"). `local:derived` is computed from other fields
(dps from damage properties, links and sockets from the sockets array) —
how the site computes it is NOT in the capture. `none` is market state a
private stash has no analogue for. `open` is everything this capture does
not settle: the item-facts census decides.
"""
import csv, json
from pathlib import Path
root = Path(__file__).resolve().parent.parent
g = json.load(open(root / "data" / "grammar.json"))
prop = {v: k for k, v in g["property_type_to_field"]["table"].items()}
FIELD = {  # filter id -> item JSON field the renderer reads (PoE/Item/Popup)
    "rarity": "frameType", "identified": "identified", "corrupted": "corrupted", "mirrored": "duplicated",
    "split": "split", "fractured_item": "fractured", "synthesised_item": "synthesised", "searing_item": "searing",
    "tangled_item": "tangled", "veiled": "veiled", "foil_variation": "foilVariation", "ilvl": "ilvl",
    "crafted": "craftedMods (non-empty)", "gem_vaal": "typeLine (Vaal prefix)?",
}
DERIVED = {"dps", "pdps", "edps", "damage", "sockets", "links"}
NONE = {"status_filters", "trade_filters"}
rows = []
for grp in g["filter_groups"]:
    for f in grp["filters"]:
        fid = f["id"]
        if grp["id"] in NONE:
            gap, via = "none", "market state (listing, seller, price, indexing)"
        elif fid in prop:
            gap, via = f"local:property {prop[fid]}", "the site renders this property type under this field"
        elif fid in FIELD:
            gap, via = "local:field", FIELD[fid]
        elif fid in DERIVED:
            gap, via = "local:derived", "computation not in the capture"
        elif fid == "category":
            gap, via = "open", "83 option ids; the private-API item carries no class field — taxonomy source unknown"
        else:
            gap, via = "open", "no property or field mapping in the capture"
        rows.append([grp["id"], fid, f.get("text") or "", "+".join(f["shape"]) or ("knownItem" if "option_raw" in f else ""), gap, via])
with open(root / "data" / "gap.csv", "w", newline="") as fh:
    w = csv.writer(fh, lineterminator="\n"); w.writerow(["group", "filter", "text", "shape", "gap", "via"]); w.writerows(rows)
from collections import Counter
by = {}
for r in rows:
    by.setdefault(r[0], Counter())[r[4].split(":")[0]] += 1
for k, c in by.items():
    print(f"{k:20s} {sum(c.values()):3d}  " + "  ".join(f"{a}={b}" for a, b in sorted(c.items())))
print("total", Counter(r[4].split(":")[0] for r in rows))
