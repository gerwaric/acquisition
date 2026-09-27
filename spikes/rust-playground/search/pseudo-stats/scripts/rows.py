#!/usr/bin/env python3
"""The rows under test: what each pseudo is believed to count, by version.

One maker for the two scripts that need it: search-sheet.py writes a
pseudo's checks from its rows, evidence.py reads every capture against them.
A pseudo's rows start as what the build ships
(../data/totals-v1.toml, the table as shipped then) or, for a pseudo no table
ships, as the lines that display its own text; each later version is one
change, with the capture that asked for it. Versions are appended, never
edited: a search already captured was composed from the version it names, and
editing that version would turn the capture into an answer to no question.

    rows.py    # print every pseudo's latest rows and the ids behind them

A row is a template, a weight and which of the stats displaying that text it
means: `any`, or the twin the site marks `(Local)` — a weapon's own damage —
or the other, `global`. `never` lists single ids that display a row's text and
are not counted. The site tells two stats with one text apart; a private item
shows the text alone, so a row that names a twin is one the build answers by
what the item is, and a `never` id is a difference it can only state.

A pseudo `reads` each line's first number (`slot`), or, for the ranged family,
the average of its two (`avg`): the site shows `Adds 54.5 to 54.5 Fire Damage
to Attacks` over a line of 31 to 78 (d05).
"""

import json
import re
import sys
import tomllib
from collections import defaultdict
from fractions import Fraction
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
ROOT = TRACK.parent.parent
STATS = TRACK.parent / "trade-query" / "data" / "stats-2026-09-12.json"
TOTALS = TRACK / "data" / "totals-v1.toml"

TYPES = ["physical", "lightning", "cold", "fire", "chaos"]
# The types the site lists a plain `… to Spells and Attacks` stat for.
BOTH = ["lightning", "cold", "fire"]
SCOPES = {"": "", "_to_attacks": " to Attacks", "_to_spells": " to Spells"}

# Every change to a pseudo's rows after its first version, in order.
# add: (template, weight); never: a stat id; why: the capture that asked.
CHANGES = {
    "total_life": [
        {
            "first": [
                ("# to maximum Life", "1"),
                ("# to Strength", "0.5"),
                ("# to Strength and Dexterity", "0.5"),
                ("# to Strength and Intelligence", "0.5"),
                ("# to all Attributes", "0.5"),
            ],
            "why": "q4 (life, Strength); p1 (the hybrid attribute lines); c1 found no item outside them",
        },
        {
            "never": ["explicit.stat_2543977012"],
            "why": "c3: `+# to Strength and Intelligence` under this id, on That Which Was "
                   "Taken, shows no total on five items of five",
        },
    ],
    "total_attack_speed": [
        {
            "add": [("While a Unique Enemy is in your Presence, #% increased Attack Speed", "1")],
            "why": "c6: ten items of ten show the total with this implicit as their one speed line",
        },
        {
            "add": [("While a Pinnacle Atlas Boss is in your Presence, #% increased Attack Speed", "1")],
            "why": "d03: 18 found, the ten fetched each showing this implicit as the total",
        },
    ],
}


TWIN = {
    "never": ["explicit.stat_2543977012"],
    "why": "e053, e057: `+# to Strength and Intelligence` under this id, on That Which Was "
           "Taken, shows no total on ten items of ten — as c3 found of total life",
}
ALL_RESISTANCES = "e042, e044, e048: one item's `+13% to All Resistances` shows as 13 under "
for _name, _weight, _seen in (("total_fire_res", "1", "fire"), ("total_lightning_res", "1", "lightning"),
                              ("total_chaos_res", "1", "chaos"), ("total_cold_res", "1", None),
                              ("total_ele_res", "3", None), ("total_res", "4", None)):
    CHANGES[_name] = [{
        "add": [("#% to All Resistances", _weight)],
        "why": ALL_RESISTANCES + ("the fire, lightning and chaos totals" if _seen else
                                  "the fire, lightning and chaos totals; this total by the "
                                  "same arithmetic, unseen"),
    }]
CHANGES["total_increased_phys"] = [{
    "never": ["enchant.stat_1509134228"],
    "why": "e061: 217 found, the ten fetched each carrying this id as the enchant `No Physical "
           "Damage`, a text that is no row's; the id is left out of what the searches ask",
}]
# Applied after the eldritch forms, so that the version round four pinned stands.
LATER = {
    "total_increased_phys": [{
        "restore": ["enchant.stat_1509134228"],
        "why": "f026: the same id displays `15% increased Physical Damage` on two items and is "
               "counted. The id carries two texts; the text decides, and `No Physical Damage` "
               "is no row's. Leaving the id out was wrong",
    }],
}
CHANGES["total_str"] = [TWIN]
CHANGES["total_int"] = [TWIN]
CHANGES["total_skill_gem_levels"] = [{
    "remove": [("# to Level of Socketed Skill Gems", "1")],
    "why": "e103: 10,000 found carrying the pseudo's own text and no total, the ten fetched one "
           "unique, Edge of Madness; e102 found none showing the total without a row",
}]


def ranged_changes():
    """The ranged family's second versions, from round two (d04–d24).

    What the captures show, one reading for all five types: a weapon's own
    line (the `(Local)` twin) feeds the attacks' pseudo and never the spells';
    the other twin feeds both; `to Spells and Attacks` feeds both; and the
    plain pseudo is complete on its own text (d04, d13, d16, d19, d22 found
    nothing). An aggregate is its types' rows together."""
    out = {}

    def scoped(kind, scope):
        text = f"Adds # to # {kind.capitalize()} Damage"
        rows = [(text, "1", "local" if scope == "_to_attacks" else "global")]
        if kind in BOTH:
            rows.append((f"{text} to Spells and Attacks", "1", "any"))
        if scope == "_to_attacks":
            rows.append((text, "1", "global"))
        return rows

    for scope in ("_to_attacks", "_to_spells"):
        for kind in TYPES:
            out[f"adds_{kind}{scope}"] = [{
                "add": scoped(kind, scope),
                "why": "round two: the items fetched by this pseudo's complete check",
            }]
        for name, kinds in (("elemental", ["fire", "cold", "lightning"]), ("damage", TYPES)):
            key = f"adds_{name}{scope}" if name != "damage" else f"adds_damage{scope}"
            add = []
            for kind in kinds:
                add.append((f"Adds # to # {kind.capitalize()} Damage{SCOPES[scope]}", "1", "any"))
                add += scoped(kind, scope)
            out[key] = [{"add": add, "why": "round two: an aggregate is its types' rows together"}]
    for name, kinds in (("elemental", ["fire", "cold", "lightning"]), ("damage", TYPES)):
        key = f"adds_{name}" if name != "damage" else "adds_damage"
        out[key] = [{
            "add": [(f"Adds # to # {kind.capitalize()} Damage", "1", "any") for kind in kinds],
            "why": "round two: an aggregate is its types' rows together",
        }]
    return out


def norm(text):
    """A stat's text as a template: the twin's suffix and the sign dropped."""
    text = re.sub(r" \((Local|Global)\)$", "", text)
    return text.replace("+#", "#").lstrip("+")


def stats():
    """({template: [id, …]}, {pseudo id: text}); the ids of `(Local)` twins are LOCAL."""
    groups = json.loads(STATS.read_text())["result"]
    ids = defaultdict(list)
    pseudo = {}
    for g in groups:
        for e in g["entries"]:
            if g["id"] == "pseudo":
                pseudo[e["id"]] = e["text"]
            else:
                ids[norm(e["text"])].append(e["id"])
                if e["text"].endswith(" (Local)"):
                    LOCAL.add(e["id"])
    return ids, pseudo


LOCAL = set()


def means(which, stat):
    """Whether a row that means `which` twin counts a line under this id."""
    return which == "any" or (which == "local") == (stat in LOCAL)


def first_versions():
    """Each pseudo's first rows: the shipped table, then the ranged family's own text."""
    out = {}
    for total in tomllib.loads(TOTALS.read_text())["total"]:
        out[total["name"]] = {
            "pseudo": "pseudo." + total["site"],
            "rows": [(r["template"], str(r["weight"])) for r in total["rows"]],
            "why": "the shipped table, totals-v1.toml",
        }
    for scope, words in SCOPES.items():
        for kind in TYPES:
            out[f"adds_{kind}{scope}"] = {
                "pseudo": f"pseudo.pseudo_adds_{kind}_damage{scope}",
                "rows": [(f"Adds # to # {kind.capitalize()} Damage{words}", "1")],
                "why": "the pseudo's own text; no source names a contributor",
            }
        for kind in ("elemental", ""):
            name = f"adds_{kind}{scope}" if kind else f"adds_damage{scope}"
            site = f"pseudo.pseudo_adds_{kind + '_' if kind else ''}damage{scope}"
            out[name] = {"pseudo": site, "rows": [], "why": "no line displays the pseudo's text"}
    out["total_life"] = {"pseudo": "pseudo.pseudo_total_life", "rows": [], "why": ""}
    return out


# Round three (e016–e107): the site counts a line's two eldritch forms as the
# line. Every complete check that found items found these, for resistances,
# speeds, damage and the ranged family alike, and nothing else in the ten
# fetched; so the rule is one, applied to every row whose form the site lists.
PRESENCE = ["While a Unique Enemy is in your Presence, ",
            "While a Pinnacle Atlas Boss is in your Presence, "]
PLAIN = [f"adds_{kind}" for kind in TYPES] + ["adds_elemental", "adds_damage"]


def round_three_changes(name, rows, ids):
    """The versions round three's captures ask for, after a pseudo's others."""
    out = []
    if name in PLAIN and any(which == "any" for (_, which) in rows):
        out.append({
            "remove": [(t, "1", "any") for (t, which) in rows if which == "any"],
            "add": [(t, "1", "global") for (t, which) in rows if which == "any"],
            "why": "e016, e021, e027, e033: items carrying a weapon's own line show no plain "
                   "pseudo; the plain pseudo counts the other twin alone",
        })
        rows = {(t, "global" if which == "any" else which): w for (t, which), w in rows.items()}
    add = [(prefix + t, str(w), "any") for (t, _), w in rows.items() for prefix in PRESENCE
           if not t.startswith("While a ") and ids.get(prefix + t)
           and (prefix + t, "any") not in rows]
    seen = []
    add = [r for r in add if not (r[0] in seen or seen.append(r[0]))]
    if add:
        out.append({"add": add, "why": "round three: the site counts a row's eldritch forms as the row"})
    return out


def row(entry):
    """(template, which twin) and the weight of a row written (template, weight[, which])."""
    return (entry[0], entry[2] if len(entry) > 2 else "any"), Fraction(entry[1])


def versions():
    """{name: [version, …]}; a version is {pseudo, reads, rows, never, why}, its
    rows {(template, which twin): weight}."""
    out = {}
    every = dict(CHANGES)
    every.update(ranged_changes())
    ids, _ = stats()
    for name, first in first_versions().items():
        rows = dict(row(r) for r in first["rows"])
        never = []
        history = []
        changes = list(every.get(name, []))
        if changes and "first" in changes[0]:
            rows = dict(row(r) for r in changes[0]["first"])
            first = dict(first, why=changes[0]["why"])
            changes = changes[1:]
        reads = "avg" if name.startswith("adds_") else "slot"
        history.append({"pseudo": first["pseudo"], "reads": reads, "rows": dict(rows), "never": [],
                        "why": first["why"]})
        def apply(change):
            nonlocal never
            for r in change.get("remove", []):
                del rows[row(r)[0]]
            for r in change.get("add", []):
                key, weight = row(r)
                rows[key] = weight
            never = [i for i in never + change.get("never", [])
                     if i not in change.get("restore", [])]
            history.append({"pseudo": first["pseudo"], "reads": reads, "rows": dict(rows),
                            "never": list(never), "why": change["why"]})

        for change in changes:
            apply(change)
        for change in round_three_changes(name, dict(rows), ids):
            apply(change)
        for change in LATER.get(name, []):
            apply(change)
        out[name] = history
    return out


def weight_of(version, template, stat):
    """The weight a version gives a line, by its template and, where the line
    carries one, its id; None where no row counts it."""
    if stat and stat in version["never"]:
        return None
    for (t, which), weight in version["rows"].items():
        if t == template and (not stat or means(which, stat)):
            return weight
    return None


def latest():
    """{pseudo id: the latest version}, for reading a capture."""
    return {h[-1]["pseudo"]: h[-1] for h in versions().values()}


def ids_of(version, ids):
    """Every counted id behind a version's rows, in every category."""
    out = []
    for (t, which) in version["rows"]:
        if not ids.get(t):
            raise SystemExit(f"no stat displays {t!r}")
        out += [i for i in ids[t]
                if i not in version["never"] and means(which, i) and i not in out]
    return out


def plain(value):
    return str(value.numerator) if value.denominator == 1 else str(float(value))


def main():
    if sys.argv[1:]:
        print(__doc__)
        return 2
    ids, pseudo = stats()
    for name, history in versions().items():
        v = history[-1]
        if v["pseudo"] not in pseudo:
            raise SystemExit(f"{name}: the site has no {v['pseudo']}")
        print(f"{name} v{len(history)} · {pseudo[v['pseudo']]} · {len(v['rows'])} rows, "
              f"{len(ids_of(v, ids))} ids" + (f", never {', '.join(v['never'])}" if v["never"] else ""))
        for (t, which), w in v["rows"].items():
            print(f"    {plain(w):>4}  {t}" + ("" if which == "any" else f"  ({which})"))
    print(f"{len(versions())} pseudos under test")
    return 0


if __name__ == "__main__":
    sys.exit(main())
