#!/usr/bin/env python3
"""Score each stated base-defence-percentile formula against the site's own
value: writes ../data/percentile-check.csv and prints the counts.

    percentile.py

One row per captured item carrying `extended.base_defence_percentile`
(search/trade-query/data/fetch-census.json: 30 of 70). For each: the base and
its defence ranges (search/repoe/data/base-defences.csv), the displayed
defences and quality (the item's `properties`), the lines read as local (the
line's trade id -> search/repoe/data/trade-stat-map.csv -> its export stat,
`is_local` in poe1/data/stats.json @ e2bd511a), the site's value, and what each
formula gives, as the lowest and the highest it could give: a displayed
defence is a rounded number.

The formulas, each as its source states it; none is preferred here:

- `apt` — awakened-poe-trade @ ce551eb7: renderer/src/parser/calc-q20.ts
  L116-L127 (calcPropPercentile; calcFlat L156-L160) and
  renderer/src/parser/Parser.ts L1266-L1282 ("Base percentile is the same for
  all defences": the first of armour, evasion, energy shield, ward the item
  carries): round(100 * (D / (1 + q/100) / (1 + inc/100) - flat - min) /
  (max - min)), clamped to 0..100. Evaluated at D - 0.5 and D + 0.5.
- `pob` — PathOfBuilding @ 16de4b82, src/Classes/Item.lua L2346-L2385: the
  same quotient per defence type, clamped to 0..1, four places; one value
  per type and no rule joining them. Its per-type columns are evaluated at
  D - 0.5 and D + 0.5; `pob_avg` joins them by the site's rule below.
- `site` — the site's own tip for the filter
  (search/trade-query/data/grammar.json, filter_groups[3].filters[5]): "The
  percentile of all base defence rolls, averaged (0-100%)". A roll is an
  integer in the base's range; the rolls are recovered as every integer B
  whose display, round-half-up of (B + flat) * (1 + inc/100) * (1 + q/100),
  equals D; each roll's percentile is 100 * (B - min) / (max - min); the
  average is rounded half up. `site_lo`/`site_hi` span every combination of
  recovered rolls; one value means the display fixes it.

Negative controls, printed: the same recovery with the display floored or
ceiled instead (how many items then admit no roll at all); the hybrids
joined by min, max or the first type instead of the average; the average
rounded half to even instead of half up; the local lines, or the quality,
not read.

The quality multiplier both tools state is also checked against the site's
own figures: `extended.ar|ev|es` "includes base value, local modifiers, and
maximum quality" (grammar.json filter_groups[3].filters[0-2]), so the
recovered roll, displayed at quality max(20, q) (APT calc-q20.ts L52-L59),
should give it exactly.
"""

import csv
import itertools
import json
import math
import re
import sys
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
SEARCH = TRACK.parent
CLONES = SEARCH.parents[3]
OUT = TRACK / "data" / "percentile-check.csv"

FETCH = SEARCH / "trade-query/data/fetch-census.json"
BASES = SEARCH / "repoe/data/base-defences.csv"
TRADE_MAP = SEARCH / "repoe/data/trade-stat-map.csv"
STATS = CLONES / "poe1/data/stats.json"

TYPES = ["ar", "ev", "es", "ward"]
PROPERTY = {"Armour": "ar", "Evasion Rating": "ev", "Energy Shield": "es", "Ward": "ward"}
BASE_COLUMNS = {
    "ar": ("armour_min", "armour_max"),
    "ev": ("evasion_min", "evasion_max"),
    "es": ("energy_shield_min", "energy_shield_max"),
    "ward": ("ward_min", "ward_max"),
}

# The export's local defence stats and what each does to which type: a flat
# addition before the increase, or an increase (the roles both tools use:
# APT calc-q20.ts L5-L38; PoB Item.lua L2347-L2365).
ROLES = {
    "local_base_physical_damage_reduction_rating": ("flat", ["ar"]),
    "local_base_evasion_rating": ("flat", ["ev"]),
    "local_energy_shield": ("flat", ["es"]),
    "local_ward": ("flat", ["ward"]),
    "local_evasion_rating_and_energy_shield": ("flat", ["ev", "es"]),
    "local_physical_damage_reduction_rating_+%": ("inc", ["ar"]),
    "local_evasion_rating_+%": ("inc", ["ev"]),
    "local_energy_shield_+%": ("inc", ["es"]),
    "local_ward_+%": ("inc", ["ward"]),
    "local_armour_and_evasion_+%": ("inc", ["ar", "ev"]),
    "local_armour_and_energy_shield_+%": ("inc", ["ar", "es"]),
    "local_evasion_and_energy_shield_+%": ("inc", ["ev", "es"]),
    "local_armour_and_evasion_and_energy_shield_+%": ("inc", ["ar", "ev", "es"]),
}
NO_QUALITY = "local_quality_does_not_increase_defences"


def read_commented_csv(path):
    with path.open(newline="\n") as fh:
        next(fh)
        reader = csv.DictReader(fh)
        return [(reader.line_num + 1, row) for row in reader]


def number(text):
    m = re.search(r"-?\d+(?:\.\d+)?", text)
    value = float(m.group()) if m else 0.0
    if " reduced " in text and value > 0:
        value = -value
    return value


def half_up(x):
    return math.floor(x + 0.5)


def half_even(x):
    return round(x)


def quotient(d, q, inc, flat, lo, hi):
    """The quotient both tools compute: the base roll's fraction of its range."""
    return (d / (1 + q / 100) / (1 + inc / 100) - flat - lo) / (hi - lo)


def apt_value(d, q, inc, flat, lo, hi):
    return min(max(half_up(quotient(d, q, inc, flat, lo, hi) * 100), 0), 100)


def pob_value(d, q, inc, flat, lo, hi):
    return round(min(max(quotient(d, q, inc, flat, lo, hi), 0), 1), 4) * 100


DISPLAY = {
    "half-up": half_up,
    "floor": lambda x: math.floor(x + 1e-9),
    "ceil": lambda x: math.ceil(x - 1e-9),
}


def rolls(d, q, inc, flat, lo, hi, display="half-up"):
    """Every integer roll whose display equals d."""
    show = DISPLAY[display]
    return [b for b in range(int(lo), int(hi) + 1)
            if show((b + flat) * (1 + inc / 100) * (1 + q / 100)) == d]


def pct(b, lo, hi):
    return 100 * (b - lo) / (hi - lo)


def fmt(x):
    if x is None:
        return ""
    return ("%.2f" % x).rstrip("0").rstrip(".")


def read_item(item, bases, tmap, local):
    shown, quality = {}, 0.0
    for prop in item.get("properties", []):
        name, values = prop[1], prop[2]
        if name == "Quality":
            quality = number(values[0])
        elif name in PROPERTY:
            shown[PROPERTY[name]] = number(values[0])
    base_row, base = None, None
    for lineno, row in bases.get(item["typeLine"], []):
        if {t for t in TYPES if row[BASE_COLUMNS[t][0]]} == set(shown):
            base_row, base = lineno, row
    flat = {t: 0.0 for t in TYPES}
    inc = {t: 0.0 for t in TYPES}
    read, no_quality = [], False
    for array, lines in item["lines"].items():
        if array == "pseudoMods":
            continue
        for line in lines:
            for stat in tmap.get(line["hash"].removeprefix("stat."), []):
                if stat == NO_QUALITY:
                    no_quality = True
                    read.append(line["description"])
                elif stat in ROLES and stat in local:
                    role, types = ROLES[stat]
                    for t in types:
                        (flat if role == "flat" else inc)[t] += number(line["description"])
                    read.append(line["description"])
    ranges = {}
    if base:
        for t in TYPES:
            lo_col, hi_col = BASE_COLUMNS[t]
            if base[lo_col] and t in shown:
                ranges[t] = (float(base[lo_col]), float(base[hi_col]))
    q = 0.0 if no_quality else quality
    return shown, quality, q, flat, inc, ranges, base_row, read


def site_values(shown, q, flat, inc, ranges, display="half-up", join=None, rounding=half_up):
    per = {}
    for t, (lo, hi) in ranges.items():
        per[t] = [pct(b, lo, hi) for b in rolls(shown[t], q, inc[t], flat[t], lo, hi, display)]
    join = join or (lambda xs: sum(xs) / len(xs))
    out = set()
    for combo in itertools.product(*per.values()):
        out.add(rounding(join(combo)))
    return per, out


def main():
    fetch = json.loads(FETCH.read_text())
    bases = {}
    for lineno, row in read_commented_csv(BASES):
        bases.setdefault(row["name"], []).append((lineno, row))
    tmap = {}
    for _, row in read_commented_csv(TRADE_MAP):
        tmap.setdefault(row["trade_id"], []).append(row["repoe_ids"])
    local = {k for k, v in json.loads(STATS.read_text()).items() if v.get("is_local")}

    header = [
        "locator", "base", "base_row", "quality", "armour", "evasion", "energy_shield", "ward",
        "local_lines", "site_value", "site_q20", "q20_predicted",
        "apt_type", "apt_point", "apt_lo", "apt_hi",
        "pob_ar_lo", "pob_ar_hi", "pob_ev_lo", "pob_ev_hi", "pob_es_lo", "pob_es_hi",
        "pob_ward_lo", "pob_ward_hi", "pob_avg_lo", "pob_avg_hi",
        "site_rolls", "site_lo", "site_hi",
        "apt_exact", "apt_within", "pob_avg_within", "site_within",
    ]
    rows, controls = [], []
    for qid, query in fetch["queries"].items():
        for i, item in enumerate(query["fetched"]):
            ext = item.get("extended", {})
            if "base_defence_percentile" not in ext:
                continue
            value = ext["base_defence_percentile"]
            shown, quality, q, flat, inc, ranges, base_row, read = read_item(item, bases, tmap, local)
            q20_type = next((t for t in ("ar", "ev", "es") if t in shown and t in ext), None)
            q20_pred = None
            if q20_type and q20_type in ranges:
                found = rolls(shown[q20_type], q, inc[q20_type], flat[q20_type], *ranges[q20_type])
                if len(found) == 1:
                    q20_pred = half_up((found[0] + flat[q20_type]) * (1 + inc[q20_type] / 100)
                                       * (1 + max(20, q) / 100))
            apt_type = next((t for t in TYPES if t in ranges), None)
            apt = (None, None, None)
            if apt_type:
                args = (q, inc[apt_type], flat[apt_type]) + ranges[apt_type]
                d = shown[apt_type]
                apt = (apt_value(d, *args), apt_value(d - 0.5, *args), apt_value(d + 0.5, *args))
            pob = {}
            for t in TYPES:
                if t in ranges:
                    args = (q, inc[t], flat[t]) + ranges[t]
                    pob[t] = (pob_value(shown[t] - 0.5, *args), pob_value(shown[t] + 0.5, *args))
            pob_avg = None
            if pob:
                pob_avg = (sum(v[0] for v in pob.values()) / len(pob),
                           sum(v[1] for v in pob.values()) / len(pob))
            per, site = site_values(shown, q, flat, inc, ranges)
            recovered = ";".join(
                f"{t}={'|'.join(str(b) for b in rolls(shown[t], q, inc[t], flat[t], *ranges[t]))}"
                for t in ranges
            )
            controls.append((value, shown, q, flat, inc, ranges, len(ranges) > 1))
            rows.append([
                f"queries.{qid}.fetched[{i}]", item["typeLine"],
                "" if base_row is None else f"base-defences.csv row {base_row}",
                fmt(quality), fmt(shown.get("ar")), fmt(shown.get("ev")), fmt(shown.get("es")),
                fmt(shown.get("ward")), "; ".join(read), value,
                ext.get(q20_type, "") if q20_type else "", fmt(q20_pred),
                apt_type or "", fmt(apt[0]), fmt(apt[1]), fmt(apt[2]),
                *[fmt(pob[t][k]) if t in pob else "" for t in TYPES for k in (0, 1)],
                fmt(pob_avg[0]) if pob_avg else "", fmt(pob_avg[1]) if pob_avg else "",
                recovered,
                fmt(min(site)) if site else "", fmt(max(site)) if site else "",
                "yes" if apt_type and apt[0] == value else "no",
                "yes" if apt_type and apt[1] <= value <= apt[2] else "no",
                "yes" if pob_avg and pob_avg[0] - 0.5 <= value <= pob_avg[1] + 0.5 else "no",
                "yes" if value in site else "no",
            ])
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with OUT.open("w", newline="") as fh:
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(header)
        w.writerows(rows)

    n = len(rows)
    col = {h: k for k, h in enumerate(header)}
    yes = lambda name: sum(1 for r in rows if r[col[name]] == "yes")
    zero = sum(1 for r in rows if r[col["site_value"]] == 0)
    hybrids = sum(1 for c in controls if c[6])
    single = sum(1 for r in rows if r[col["site_lo"]] != "" and r[col["site_lo"]] == r[col["site_hi"]])
    q20 = [(float(r[col["q20_predicted"]]), float(r[col["site_q20"]])) for r in rows if r[col["site_q20"]] != ""]
    q20_ok = sum(1 for p, s in q20 if p == s)
    print(f"{n} items carry the site's value, {zero} of them 0; {hybrids} bases carry two defence types")
    print(f"apt, exact at the displayed value:        {yes('apt_exact')} of {n}")
    print(f"apt, within its rounding envelope:        {yes('apt_within')} of {n}")
    print(f"pob per type, averaged (the site's rule): {yes('pob_avg_within')} of {n}")
    print(f"site's rule over recovered integer rolls: {yes('site_within')} of {n}"
          f" ({single} of {n} a single value: the display fixes it)")
    print(f"site's 20%-quality figure from the recovered roll, exactly: {q20_ok} of {len(q20)}")
    print("negative controls (the site's rule, one part changed):")
    for display in ("floor", "ceil"):
        none_ = sum(1 for c in controls if not site_values(*c[1:6], display=display)[1])
        hit = sum(1 for c in controls if c[0] in site_values(*c[1:6], display=display)[1])
        print(f"  display {display:5}: {none_} items admit no roll; {hit} of {n} reproduced")
    for name, join in (("min", min), ("max", max), ("first type", lambda xs: xs[0])):
        hit = sum(1 for c in controls if c[6] and c[0] in site_values(*c[1:6], join=join)[1])
        print(f"  hybrids joined by {name:10}: {hit} of {hybrids} reproduced (average: "
              f"{sum(1 for c in controls if c[6] and c[0] in site_values(*c[1:6])[1])})")
    hit = sum(1 for c in controls if c[0] in site_values(*c[1:6], rounding=half_even)[1])
    print(f"  average rounded half to even: {hit} of {n} reproduced")
    with_lines = sum(1 for c in controls if any(c[3].values()) or any(c[4].values()))
    zero = {t: 0.0 for t in TYPES}
    hit = sum(1 for c in controls if (any(c[3].values()) or any(c[4].values()))
              and c[0] in site_values(c[1], c[2], zero, zero, c[5])[1])
    print(f"  local lines not read: {hit} of the {with_lines} items carrying one reproduced")
    hit = sum(1 for c in controls if c[2] and c[0] in site_values(c[1], 0.0, c[3], c[4], c[5])[1])
    print(f"  quality not read: {hit} of the {sum(1 for c in controls if c[2])} items with quality reproduced")
    print(f"wrote {OUT.relative_to(SEARCH.parent)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
