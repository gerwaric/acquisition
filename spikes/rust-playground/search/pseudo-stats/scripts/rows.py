#!/usr/bin/env python3
"""The rows under test: what each pseudo is believed to count, by version.

One maker for the two scripts that need it: search-sheet.py writes a
pseudo's checks from its rows, evidence.py reads every capture against them.
A pseudo's rows start as what the build ships
(crates/acquisition-search/reference/totals-v1.toml) or, for a pseudo no table
ships, as the lines that display its own text; each later version is one
change, with the capture that asked for it. Versions are appended, never
edited: a search already captured was composed from the version it names, and
editing that version would turn the capture into an answer to no question.

    rows.py    # print every pseudo's latest rows and the ids behind them

A row is a template and a weight. `never` lists stat ids that display a row's
text and are not counted: the site tells two stats with one text apart, a
private item cannot, so an id here is a difference the search can state and
cannot mimic by the line alone.
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
TOTALS = ROOT / "crates" / "acquisition-search" / "reference" / "totals-v1.toml"

TYPES = ["physical", "lightning", "cold", "fire", "chaos"]
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
    ],
}


def norm(text):
    """A stat's text as a template: the twin's suffix and the sign dropped."""
    text = re.sub(r" \((Local|Global)\)$", "", text)
    return text.replace("+#", "#").lstrip("+")


def stats():
    groups = json.loads(STATS.read_text())["result"]
    ids = defaultdict(list)
    pseudo = {}
    for g in groups:
        for e in g["entries"]:
            if g["id"] == "pseudo":
                pseudo[e["id"]] = e["text"]
            else:
                ids[norm(e["text"])].append(e["id"])
    return ids, pseudo


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


def versions():
    """{name: [version, …]}, a version {pseudo, rows: {template: weight}, never, why}."""
    out = {}
    for name, first in first_versions().items():
        rows = {t: Fraction(w) for t, w in first["rows"]}
        never = []
        history = []
        changes = list(CHANGES.get(name, []))
        if changes and "first" in changes[0]:
            rows = {t: Fraction(w) for t, w in changes[0]["first"]}
            first = dict(first, why=changes[0]["why"])
            changes = changes[1:]
        history.append({"pseudo": first["pseudo"], "rows": dict(rows), "never": [], "why": first["why"]})
        for change in changes:
            for t, w in change.get("add", []):
                rows[t] = Fraction(w)
            for t in change.get("remove", []):
                del rows[t]
            never = never + change.get("never", [])
            history.append({"pseudo": first["pseudo"], "rows": dict(rows), "never": list(never),
                            "why": change["why"]})
        out[name] = history
    return out


def latest():
    """{pseudo id: the latest version}, for reading a capture."""
    return {h[-1]["pseudo"]: h[-1] for h in versions().values()}


def ids_of(version, ids):
    """Every counted id behind a version's rows, in every category."""
    out = []
    for t in version["rows"]:
        if not ids.get(t):
            raise SystemExit(f"no stat displays {t!r}")
        out += [i for i in ids[t] if i not in version["never"]]
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
        for t, w in v["rows"].items():
            print(f"    {plain(w):>4}  {t}")
    print(f"{len(versions())} pseudos under test")
    return 0


if __name__ == "__main__":
    sys.exit(main())
