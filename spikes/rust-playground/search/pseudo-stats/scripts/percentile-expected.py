#!/usr/bin/env python3
"""The percentile's expected answers (the build plan, 9d-A; BRIEF.md): writes
../data/percentile-expected.csv and ../data/percentile-expected-copy.csv and
prints every count the track's README states of them.

    percentile-expected.py                 # write both files, print the counts
    percentile-expected.py --break NAME    # the reading with one rule broken: print how many
                                           # captured cells then disagree, write nothing
    percentile-expected.py --breaks        # list the breaks

One function, `read`, goes from what a private item carries — its realm, its
base, its rarity, its properties as displayed, each line's text, array and
flags, and the item's own marks (`searing`, `tangled`, `synthesised`) — to what
`pseudo.defence_pct` answers of it: `lacked`, a value, or `undecided` with its
cause; `unruled` is kept for a cell the rulings give two readings of, and no
cell is one since J33 to J35 (2026-10-08). Beside the answer, what it read: each defence type's display,
range, flat and increase, the quality term, the rolls the display admits and
each roll's percentile, the lines taken and the lines left out. It never reads
a line's `hash` or `mods`, nor `extended`: the site's value and the site's id
are read beside it (`site_value`, `site_side`), and nothing in `read` takes them.

Every step of `read` is a key of STEPS, with the sentences it carries out
(CITES); a step with nothing to cite would be a key of UNRULED and its cell
`unruled`, and there is none. The ranges are ../repoe/data/base-defences.csv's, read and never
typed. Which stat a line's text is: percentile-shapes.py's join (`role_ids`),
the trade site's `(Local)` texts to the export's local defence stats, the
`reduced` spelling the same stat below nothing; which of those texts a global
stat displays too (a twin text): defence-twins.py's (`twin_texts`). The realms
with a table are the shipped class table's, made from the same export as the
ranges. A captured item's marks are read as defence-twins.py reads them, from
the fetch it came from (raw/searches/<search>-fetch*.json, local only, the n-th
item of a search's pages the extract's n-th, checked by base; only the three
key names leave raw/); where raw/ is absent, from data/defence-twins.csv, which
holds them for every captured item showing a defence and carrying a twin line,
and an item it does not hold has its marks written `(not kept)`. The reading
asks a mark of an implicit twin text alone; the marks of the others say which
group of lines a captured line is counted in (step 5).

Numbers are exact: a displayed number is a fraction, never a float.

Inputs: data/captures.json, raw/searches/, data/defence-twins.csv, ../trade-query/data/
fetch-census.json, ../repoe/data/base-defences.csv, ../repoe/data/
trade-stat-map.csv, crates/acquisition-search/reference/classes-v1.toml, and
the owner's store copy, read-only (../item-facts/raw/m3/store/mock/
GERWARIC_7694.db, `mode=ro`). A local input that is absent is named and the
script goes on with what is there.
"""

import csv
import importlib.util
import itertools
import json
import math
import re
import sqlite3
import sys
import tomllib
from collections import Counter, defaultdict
from fractions import Fraction
from pathlib import Path

HERE = Path(__file__).resolve().parent
_spec = importlib.util.spec_from_file_location("defence_twins", HERE / "defence-twins.py")
D = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(D)
S = D.S  # percentile-shapes.py
P = D.P  # percentile.py

TRACK = P.TRACK
SEARCH = P.SEARCH
ROOT = SEARCH.parent
CAPTURES = TRACK / "data" / "captures.json"
TWINS_CSV = TRACK / "data" / "defence-twins.csv"
RAW = TRACK / "raw" / "searches"
FETCH = SEARCH / "trade-query/data/fetch-census.json"
STORE = S.STORE
CLASSES = S.CLASSES
OUT = TRACK / "data" / "percentile-expected.csv"
OUT_COPY = TRACK / "data" / "percentile-expected-copy.csv"
INPUTS = ("data/captures.json, raw/searches/*-fetch*.json (a captured item's marks, the key names alone; "
          "where absent, data/defence-twins.csv's), "
          "../trade-query/data/fetch-census.json, ../repoe/data/base-defences.csv, "
          "../repoe/data/trade-stat-map.csv, crates/acquisition-search/reference/classes-v1.toml, "
          "../item-facts/raw/m3/store/mock/GERWARIC_7694.db (read-only)")

TYPES = P.TYPES  # ar, ev, es, ward
PROPERTY = P.PROPERTY  # the property name each type is displayed under
WORD = {"ar": "Armour", "ev": "Evasion Rating", "es": "Energy Shield", "ward": "Ward"}
MARKS = ("searing", "tangled", "synthesised")
HALF = Fraction(1, 2)

# The sentences a step of the reading carries out, cited by key.
CITES = {
    "C101-range": "DESIGN.md, C101 — the derived fields not yet built: `Base defence percentile is the item's "
                  "defence against its base's range from reference data` (base-defences.csv)",
    "C101-over": "the same paragraph: `an item above its base's current maximum reads over 100 %`; and `A roll "
                 "over the base's maximum stays unpinned: the site lists no armour over 100, so \"reads over "
                 "100 %\" above is the stated rule's and no capture's`",
    "C101-realm": "the same paragraph: `a realm with no table is undecided with that reason, never zero`",
    "C101-rule": "the same paragraph: `The percentile is the site's own stated rule — each base defence roll's "
                 "percentile, averaged, rounded half up, a roll being any integer whose display is the item's`",
    "J5": "the same paragraph and LEDGER 9d, J5: `a display at an exact half admits both integers as rolls`, "
          "the definition's one departure from the site's tip",
    "J6-three": "the same paragraph (J6, round i, i01): `three defence types on one base are the average of "
                "three rolls`",
    "J6-enchant": "the same paragraph (J6, i02): `under the enchant that zeroes quality the quality term is "
                  "dropped`",
    "J6-ward": "the same paragraph (J6, i09): `ward is a roll as the others are`",
    "J8-eldritch": "the same paragraph and LEDGER 9d, J8: `an implicit twin on an eldritch item is the global "
                   "stat and left out`",
    "J8-explicit": "J8: `an explicit twin, and an implicit twin on a synthesised item, are the item's own` — "
                   "the explicit",
    "J8-synthesised": "J8: `an explicit twin, and an implicit twin on a synthesised item, are the item's own` "
                      "— the synthesised implicit",
    "J8-unmarked": "J8: `an implicit twin on an item of neither mark, seen nowhere, cannot be established and "
                   "the field is undecided there with that reason (C93)`",
    "J32": "LEDGER 9d, J32: `a flag that is no yes or no leaves open only what asks a flag`; the reading asks "
           "a line's array and the item's marks and no flag, so the line counts by its text",
    "C93-lacked": "decisions/search.md, C93: `A comparison on something the item lacks is false, never zero`; "
                  "pseudo.rs, C101: an item with none of the properties a field reads lacks the field — known "
                  "absence; LEDGER 7, T2: `has:` asks a derived field's presence",
    "C93-open": "decisions/search.md, C93: `What cannot be established on an item — an unread body, a base the "
                "class table lacks, an unresolved price — makes only the terms that needed it undecided`",
    "doc-unread": "pseudo.rs, C101: `where a property the field reads is unread, or is displayed as no number "
                  "and no range … the field is undecided with that reason. An unread element of properties "
                  "leaves a field open only when it may be one the field reads: its name unread, or one of "
                  "the field's`",
    "rule-8": "the plan, rule 8: `What the deriver cannot read — a flag that is no yes or no, a number it does "
              "not read, an array that is no array — is unread there, at that grain`, and of no more than "
              "was lost",
    "rows-of-a-mod": "LEDGER 9c4: a mod displayed over several rows is named by a row of it and reads that "
                     "row's number",
    "tip": "README, The percentile: the site's tip `The percentile of all base defence rolls, averaged`; rolls "
           "= the integers whose display (half up) is D; the display of a roll B is (B + flat) x (1 + "
           "increase/100) x (1 + quality/100), flat before increase (percentile.py, ROLES: APT calc-q20.ts "
           "L5-L38, PoB Item.lua L2347-L2365), `quality multiplies, as both tools state`",
    "join": "percentile-shapes.py, `role_ids`: a line's template is given the role its local stat has — the "
            "trade site's `(Local)` texts joined to the export's stats, `the reduced spelling the same stat "
            "below nothing`; README, the headline: `A private item displays everything the rule reads`",
    "C9": "README, The percentile's round, C9: `a defence the base lacks (its own types read)`; a defence the "
          "base ranges none of is no base defence roll (the tip: `all base defence rolls`)",
    "one-value": "README, The percentile's round: the captured items reproduced `each one value` — the display "
                 "fixes the answer",
    "J33": "LEDGER 9d, J33 (2026-10-08): the percentile is a value only where what the item shows gives one "
           "value from lines whose side is established; anywhere else it is undecided, and the reason says "
           "which part failed",
    "J34": "LEDGER 9d, J34 (2026-10-08; the owner: `keep reads over 100`): a roll over the base's maximum "
           "reads over 100, and one under its minimum below 0",
    "J35": "LEDGER 9d, J35 (2026-10-08; the owner: `undecided`): in a realm with no table the field is "
           "undecided on every item, one displaying no defence among them",
    "realms": "crates/acquisition-search/reference/classes-v1.toml, `realms`: the realms the shipped tables of "
              "this export cover; base-defences.csv is the same export's and carries no realm line",
}

# Every step of `read`: what it does, and the sentences it carries out.
STEPS = {
    "realm": ("a realm with no table: undecided with that reason, the item displaying a defence or a property "
              "unread", ["C101-realm", "realms"]),
    "property-unread": ("a property the field reads is unread — `properties` no array, an element's name "
                        "unread, a defence or the quality displayed as no number: undecided",
                        ["doc-unread", "rule-8"]),
    "lacked": ("the item displays none of Armour, Evasion Rating, Energy Shield, Ward: lacked",
               ["C93-lacked"]),
    "base-lacking": ("the range table holds no base of the item's name, or the base is unread: undecided",
                     ["C93-open", "C101-range"]),
    "base": ("the base's row of the range table, held once", ["C101-range"]),
    "base-held-twice": ("a name the range table holds more than once: the item's base is the one row all of "
                        "whose defences the item displays", ["C101-range", "C93-open"]),
    "base-held-twice-open": ("a name held more than once, and more than one row all of whose defences the item "
                             "displays: the base is not established, undecided", ["C93-open"]),
    "not-of-base": ("a defence the item displays and its base ranges none of is no roll: not read",
                    ["C9", "C101-rule"]),
    "array-unread": ("an array of lines that is no array: undecided", ["rule-8", "C93-open"]),
    "line-unread": ("a line that is no text: undecided", ["rule-8", "C93-open"]),
    "rows-of-a-mod": ("a line displayed over several rows is read row by row", ["rows-of-a-mod"]),
    "flag-unread": ("a flag that is no yes or no: the line counts by its text", ["J32"]),
    "line-of-no-roll": ("a defence line of a type no roll reads changes no roll, whatever stat it is: not "
                        "read, and never a cause", ["rule-8", "C93-open", "C9"]),
    "local-only": ("a text only a local defence stat displays is the item's own, whatever its array",
                   ["join", "tip"]),
    "twin-explicit": ("a twin text in `explicitMods` is the item's own", ["J8-explicit"]),
    "twin-eldritch": ("a twin text in `implicitMods` of an eldritch item (`searing` or `tangled`) is the "
                      "global stat: left out", ["J8-eldritch"]),
    "twin-synthesised": ("a twin text in `implicitMods` of a synthesised item is the item's own",
                         ["J8-synthesised"]),
    "twin-unmarked": ("a twin text in `implicitMods` of an item of neither mark: undecided",
                      ["J8-unmarked", "C93-open"]),
    "mark-unread": ("a twin text in `implicitMods` of an item whose mark is no yes or no, or not kept: "
                    "undecided", ["rule-8", "J8-unmarked"]),
    "reduced": ("a `reduced` line is its stat below nothing", ["join"]),
    "flat": ("a local flat line is added to the roll before the increase", ["tip", "join"]),
    "increase": ("local increases are summed and multiply the roll and the flat", ["tip", "join"]),
    "quality": ("the displayed quality multiplies", ["tip"]),
    "enchant": ("`Quality does not increase Defences`: the quality term is dropped", ["J6-enchant"]),
    "roll": ("a roll is an integer whose display, rounded half up, is the item's; its percentile is 100 x "
             "(roll - min) / (max - min)", ["C101-rule", "tip", "C101-range"]),
    "half": ("a display at an exact half admits the integer whose display rounds half up to the next number "
             "too", ["J5"]),
    "no-roll": ("no integer gives the display of a defence the base ranges: undecided", ["C93-open",
                                                                                         "C101-rule"]),
    "roll-over-max": ("a roll over the base's maximum is a roll: its percentile is over 100",
                      ["C101-over", "J34", "C101-rule"]),
    "roll-under-min": ("a roll under the base's minimum is a roll, `any integer whose display is the item's`: "
                       "its percentile is below 0", ["J34", "C101-rule"]),
    "several-rolls-one-value": ("more than one roll gives a display, and every combination rounds to one "
                                "value: that value", ["C101-rule", "one-value"]),
    "one-type": ("one defence type: its roll's percentile, rounded half up", ["C101-rule"]),
    "two-types": ("two defence types: the two percentiles averaged, rounded half up", ["C101-rule"]),
    "three-types": ("three defence types: the three percentiles averaged, rounded half up",
                    ["J6-three", "C101-rule"]),
    "ward": ("ward is a roll as the others are", ["J6-ward"]),
    "average-at-a-half": ("an average at an exact half rounds up", ["C101-rule"]),
    "realm-no-defence": ("a realm with no table, the item displaying no defence: undecided as every item of "
                         "the realm is", ["J35", "C101-realm"]),
    "base-type-not-shown": ("no row of the base has every defence it ranges displayed: undecided",
                            ["J33", "C93-open"]),
    "twin-both-marks": ("a twin text in `implicitMods` of an item both eldritch and synthesised: undecided",
                        ["J33", "J8-unmarked"]),
    "twin-other-array": ("a twin text in an array J8 does not name, `crucibleMods` apart: undecided",
                         ["J33", "J8-unmarked"]),
    "crucible-line": ("a defence line in `crucibleMods` of a type a roll reads: undecided, the site not "
                      "asked", ["J33"]),
    "flat-range": ("a base whose minimum is its maximum: no percentile, undecided", ["J33"]),
    "several-values": ("more than one roll gives a display, and the rolls give different values: undecided",
                       ["J33"]),
}

# Where the rulings give two readings, or none: the cell is `unruled`, and carries each reading with what
# it would change. A key is a step of `read` too. None since J33 to J35.
UNRULED = {}

# One break for each rule of the reading (step 4): what the break does.
BREAKS = {
    "realm": "a realm with no table is read by the table",
    "lacked-is-zero": "an item displaying no defence answers 0, not lacked",
    "property-unread-ignored": "an unread property is read as absent",
    "base-lacking-is-lacked": "an item whose base the range table lacks is lacked",
    "held-twice-other-row": "a name the range table holds more than once is read by the row after the item's "
                            "own, the defences the two share alone",
    "base-lacks-type-undecided": "an item displaying a defence its base ranges none of is undecided",
    "array-unread-ignored": "an unread array, or a line that is no text, is read as absent",
    "whole-text": "a line displayed over several rows is read by its whole text, not by a row",
    "flag-unread-opens": "a flag that is no yes or no leaves the field undecided",
    "unflagged-only": "a line under a flag (crafted, fractured) is not read",
    "lines-not-read": "no line is read: the display is the roll times the quality",
    "local-only-explicit-only": "a text only a local stat displays is read in `explicitMods` alone",
    "explicit-twin-left-out": "a twin text in `explicitMods` is left out",
    "eldritch-twin-own": "a twin text in `implicitMods` of an eldritch item is read as the item's own",
    "synthesised-twin-left-out": "a twin text in `implicitMods` of a synthesised item is left out",
    "unmarked-twin-own": "a twin text in `implicitMods` of an item of neither mark is read as the item's own",
    "twin-of-no-roll-opens": "an implicit twin of a type no roll reads leaves the field undecided on an item "
                             "of neither mark",
    "reduced-as-increased": "a `reduced` line is read as an increase",
    "flat-after-increase": "a flat line is added after the increase",
    "increases-multiply": "each local increase multiplies on its own, not summed",
    "quality-not-read": "the quality is not read",
    "enchant-ignored": "under `Quality does not increase Defences` the quality is read after all",
    "display-floored": "a roll's display is floored, not rounded half up",
    "j5-off": "a display at an exact half admits the stated rule's integer alone",
    "range-bound": "a roll is an integer of the base's range and no other",
    "first-type-only": "the first defence type is read and no other",
    "ward-not-read": "ward is not a roll",
    "average-half-even": "the average is rounded half to even",
    "average-floored": "the average is floored",
}

NUMBER = re.compile(r"[-+]?\d+(?:\.\d+)?")


def template(text):
    return D.template(text)  # search/item-facts's rule, poe2's markup dropped


def number(text):
    """A line's first number, exact; a `reduced` line is its stat below nothing (percentile.py, `number`)."""
    m = NUMBER.search(text)
    value = Fraction(m.group()) if m else Fraction(0)
    reduced = " reduced " in text and value > 0
    return value, reduced


def plain(x):
    """A fraction as written: an integer, or two decimals."""
    if x is None:
        return ""
    x = Fraction(x)
    if x.denominator == 1:
        return str(x.numerator)
    return f"{float(x):.2f}"


def half_up(x):
    return math.floor(x + HALF)


def half_even(x):
    lower = math.floor(x)
    if x - lower == HALF:
        return lower if lower % 2 == 0 else lower + 1
    return half_up(x)


class Tables:
    def __init__(self):
        self.realms = set(tomllib.loads(CLASSES.read_text())["realms"])
        self.bases = defaultdict(list)  # name -> [(csv line, {type: (min, max)})]
        for lineno, row in P.read_commented_csv(P.BASES):
            ranges = {t: (int(row[P.BASE_COLUMNS[t][0]]), int(row[P.BASE_COLUMNS[t][1]]))
                      for t in TYPES if row[P.BASE_COLUMNS[t][0]]}
            self.bases[row["name"]].append((lineno, ranges))
        tmap = {}
        for _, row in P.read_commented_csv(P.TRADE_MAP):
            tmap.setdefault(row["trade_id"], []).append(row["repoe_ids"])
        self.roles = {}  # template -> ("flat" | "inc", [types]) or ("no-quality", [])
        for text, trade_id in S.role_ids().items():
            for stat in tmap[trade_id]:
                if stat == P.NO_QUALITY:
                    self.roles[text] = ("no-quality", [])
                elif stat in P.ROLES:
                    self.roles[text] = P.ROLES[stat]
        self.twins, self.site_side = D.twin_texts()
        self.classes = defaultdict(set)
        for c in tomllib.loads(CLASSES.read_text())["class"]:
            for b in c["bases"]:
                self.classes[b].add(c["name"])

    def class_of(self, base):
        return "|".join(sorted(self.classes.get(base, set()))) or "not in the class table"


def mark_kind(marks):
    """eldritch | synthesised | both | neither | unread — what J8 reads of the item."""
    if marks is None:
        return "unread"
    if any(not isinstance(marks.get(k), bool) for k in MARKS if k in marks):
        return "unread"
    eldritch = bool(marks.get("searing")) or bool(marks.get("tangled"))
    synthesised = bool(marks.get("synthesised"))
    if eldritch and synthesised:
        return "both"
    return "eldritch" if eldritch else "synthesised" if synthesised else "neither"


def displayed(properties):
    """({type: display} of the defences shown, the quality or None, the causes a property is unread)."""
    shown, quality, unread, quality_unread = {}, None, [], None
    if properties is None:
        return shown, quality, unread, quality_unread
    if not isinstance(properties, list):
        return shown, quality, ["`properties` is no array"], quality_unread
    for element in properties:
        name, value = element if isinstance(element, (list, tuple)) and len(element) == 2 else (None, None)
        if name is None and isinstance(element, (list, tuple)) and len(element) == 2:
            continue  # derive.rs, `property`: a name that is absent or null is an empty name, no unread one
        if not isinstance(name, str):
            unread.append("a property's name is unread")
            continue
        name = D.MARKUP.sub(lambda m: m.group(2) or m.group(1), name)
        if name in PROPERTY:
            if isinstance(value, str) and re.fullmatch(r"\d+", value):
                shown[PROPERTY[name]] = int(value)
            else:
                shown[PROPERTY[name]] = None
                unread.append(f"`{name}` is displayed as no number")
        elif name == "Quality":
            m = re.fullmatch(r"\+?(\d+)%?", value) if isinstance(value, str) else None
            if m:
                quality = int(m.group(1))
            else:
                quality_unread = "`Quality` is displayed as no number"
    return shown, quality, unread, quality_unread


def answer(status, steps, value=None, cause="", **more):
    out = {"status": status, "value": value, "cause": cause, "steps": list(dict.fromkeys(steps)),
           "types": {}, "taken": [], "left": [], "quality": "", "base_row": "", "readings": [],
           "not_of_base": [], "lines": []}
    out.update(more)
    return out


def read(tables, item, broken=None):
    """What `pseudo.defence_pct` answers of an item.

    item: {"realm", "base", "rarity", "marks": {mark: value} or None where they were not kept,
    "properties": [(name, value)] or a thing that is no list, "lines": {array: [(text, flags)] or a thing
    that is no list}}. Returns {"status": lacked | value | undecided | unruled, "value", "cause", "steps",
    "types": {type: {display, range, flat, inc, rolls: [(roll, percentile, at a half)]}}, "taken", "left",
    "quality", "base_row", "readings": [(reading, what it gives)], "lines": [each defence line with what
    became of it]}.
    """
    steps = []
    shown, quality, unread, quality_unread = displayed(item.get("properties"))
    if broken == "property-unread-ignored":
        shown = {t: d for t, d in shown.items() if d is not None}
        unread, quality_unread = [], None

    # C101: a realm with no table is undecided with that reason, never zero.
    if item["realm"] not in tables.realms and broken != "realm":
        if shown or unread:
            return answer("undecided", ["realm"], cause=f"realm: `{item['realm']}` has no table")
        # J35: every item of the realm, one displaying no defence among them
        return answer("undecided", ["realm-no-defence"], cause=f"realm: `{item['realm']}` has no table")

    # pseudo.rs, C101; rule 8: a property the field reads, unread, leaves the field open.
    if unread:
        return answer("undecided", ["property-unread"], cause="property unread: " + "; ".join(unread))
    # C93; pseudo.rs: an item with none of the properties the field reads lacks the field.
    if not shown:
        if broken == "lacked-is-zero":
            return answer("value", ["lacked"], value=0)
        return answer("lacked", ["lacked"])
    if quality_unread:
        return answer("undecided", ["property-unread"], cause="property unread: " + quality_unread)

    # C101: the item's defence against its base's range from reference data.
    base = item.get("base")
    rows = tables.bases.get(base, []) if isinstance(base, str) else []
    if not rows:
        if broken == "base-lacking-is-lacked":
            return answer("lacked", ["base-lacking"])
        cause = f"base: the range table lacks `{base}`" if isinstance(base, str) else "base: unread"
        return answer("undecided", ["base-lacking"], cause=cause)
    fits = [(n, r) for n, r in rows if r and set(r) <= set(shown)]
    if broken == "held-twice-other-row" and len(rows) > 1 and len(fits) == 1:
        after = rows[([n for n, _ in rows].index(fits[0][0]) + 1) % len(rows)]
        fits = [(after[0], {t: v for t, v in after[1].items() if t in shown})]
    if len(fits) > 1:
        # C93: more than one row could be the item's base, and nothing the item carries tells which
        return answer("undecided", ["base-held-twice-open"],
                      cause=f"base: the range table holds `{base}` {len(rows)} times, and the defences "
                            f"displayed are those of {len(fits)} of its rows")
    if not fits:
        # J33: the base's row is not the item's by what it displays
        return answer("undecided", ["base-type-not-shown"],
                      cause=f"base: no row of `{base}` has every defence it ranges displayed",
                      base_row="; ".join(f"row {n}: {'/'.join(r) or 'no defence'}" for n, r in rows))
    base_row, ranges = fits[0]
    steps.append("base-held-twice" if len(rows) > 1 else "base")
    # C9; the tip: a defence the base ranges none of is no base defence roll.
    not_of_base = [t for t in TYPES if t in shown and t not in ranges]
    if not_of_base:
        steps.append("not-of-base")
        if broken == "base-lacks-type-undecided":
            return answer("undecided", steps, cause="a defence displayed that the base lacks")
    if broken == "ward-not-read":
        ranges = {t: v for t, v in ranges.items() if t != "ward"}
        if not ranges:
            return answer("lacked", steps)
    if broken == "first-type-only":
        first = next(t for t in TYPES if t in ranges)
        ranges = {first: ranges[first]}

    # The lines: which stat a line's text is (the join), and which twin (J8).
    kind = mark_kind(item.get("marks"))
    flat = {t: Fraction(0) for t in ranges}
    incs = {t: [] for t in ranges}
    taken, left, lines, opened = [], [], [], []
    no_quality = False
    for array, entries in (item.get("lines") or {}).items():
        if not isinstance(entries, list):
            # rule 8: an array that is no array is unread, and could hold a line the field reads
            if broken != "array-unread-ignored":
                return answer("undecided", steps + ["array-unread"], cause=f"`{array}` is no array")
            continue
        for text, flags in entries:
            if not isinstance(text, str):
                if broken != "array-unread-ignored":
                    return answer("undecided", steps + ["line-unread"], cause=f"a line of `{array}` is no text")
                continue
            rows_of = [text] if broken == "whole-text" else text.split("\n")
            for row in rows_of:
                t = template(row)
                role = tables.roles.get(t)
                if role is None:
                    continue
                if len(rows_of) > 1:
                    steps.append("rows-of-a-mod")  # LEDGER 9c4
                shown_flags = ";".join(sorted(k for k, v in (flags or {}).items() if v is True))
                if any(not isinstance(v, bool) for v in (flags or {}).values()):
                    # J32: a flag that is no yes or no opens only what asks a flag, and nothing here does
                    if broken == "flag-unread-opens":
                        return answer("undecided", steps, cause="a line's flag is no yes or no")
                    steps.append("flag-unread")
                    shown_flags = (shown_flags + ";" if shown_flags else "") + "(a flag unread)"
                if broken == "unflagged-only" and any(v is True for v in (flags or {}).values()):
                    continue
                if broken == "lines-not-read":
                    continue
                line = {"array": array, "flags": shown_flags, "row": row, "template": t, "role": role[0],
                        "twin": t in tables.twins, "mark": kind}
                lines.append(line)
                if role[0] == "no-quality":
                    # J6, i02: under the enchant that zeroes quality the quality term is dropped
                    line["became"], line["rule"] = "taken: the quality term dropped", "enchant"
                    if broken != "enchant-ignored":
                        no_quality = True
                        steps.append("enchant")
                    taken.append(line)
                    continue
                feeds = [x for x in role[1] if x in ranges]
                twin = t in tables.twins
                if not feeds:
                    implicit_open = twin and array == "implicitMods" and kind == "neither"
                    if broken == "twin-of-no-roll-opens" and implicit_open:
                        return answer("undecided", steps, cause="an implicit twin on an item of neither mark")
                    # rule 8, C93: a line of a type no roll reads changes nothing, whatever stat it is
                    line["became"], line["rule"] = "not read: of a type no roll reads", "line-of-no-roll"
                    steps.append("line-of-no-roll")
                    left.append(line)
                    continue
                if array == "crucibleMods":
                    rule, became = "crucible-line", "open"  # J33
                elif not twin:
                    # the join: a text only a local stat displays is the item's own
                    rule, became = "local-only", "own"
                    if broken == "local-only-explicit-only" and array != "explicitMods":
                        became = "left"
                elif array == "explicitMods":
                    rule, became = "twin-explicit", "own"  # J8
                    if broken == "explicit-twin-left-out":
                        became = "left"
                elif array == "implicitMods":
                    if kind == "eldritch":
                        rule, became = "twin-eldritch", "left"  # J8
                        if broken == "eldritch-twin-own":
                            became = "own"
                    elif kind == "synthesised":
                        rule, became = "twin-synthesised", "own"  # J8
                        if broken == "synthesised-twin-left-out":
                            became = "left"
                    elif kind == "neither":
                        rule, became = "twin-unmarked", "open"  # J8
                        if broken == "unmarked-twin-own":
                            became = "own"
                    elif kind == "both":
                        rule, became = "twin-both-marks", "open"  # J33
                    else:
                        rule, became = "mark-unread", "open"  # rule 8
                else:
                    rule, became = "twin-other-array", "open"  # J33
                line["rule"] = rule
                steps.append(rule)
                value, reduced = number(row)
                if reduced:
                    if broken != "reduced-as-increased":
                        value = -value
                    steps.append("reduced")
                line["value"], line["feeds"] = value, feeds
                if became == "own":
                    line["became"] = f"taken: {role[0]} {'/'.join(feeds)}"
                    taken.append(line)
                elif became == "left":
                    line["became"] = "left out: the global stat"
                    left.append(line)
                else:
                    line["became"] = "open"
                    opened.append(line)

    def apply(extra):
        f = dict(flat)
        i = {t: list(v) for t, v in incs.items()}
        for line in taken + extra:
            if "value" not in line:
                continue
            for t in line["feeds"]:
                if line["role"] == "flat":
                    f[t] += line["value"]
                else:
                    i[t].append(line["value"])
        return f, i

    # The quality term: displayed, and dropped under the enchant.
    q = 0 if (no_quality or quality is None or broken == "quality-not-read") else quality
    quality_text = ("none displayed" if quality is None else
                    f"{quality}%, dropped under the enchant" if no_quality else f"{quality}%")

    def compute(extra):
        """(status, value, cause, {type: detail}, the steps taken, the values) for the lines taken + extra."""
        f, i = apply(extra)
        took, detail, per = [], {}, {}
        if any(line["role"] == "flat" for line in taken + extra if "value" in line):
            took.append("flat")
        if any(line["role"] == "inc" for line in taken + extra if "value" in line):
            took.append("increase")
        if q:
            took.append("quality")
        for t, (lo, hi) in ranges.items():
            d = shown[t]
            inc = sum(i[t], Fraction(0))
            if broken == "increases-multiply":
                factor = Fraction(1)
                for x in i[t]:
                    factor *= 1 + x / 100
            else:
                factor = 1 + inc / 100
            mult = factor * (1 + Fraction(q, 100))

            def display(b, t=t, factor=factor, mult=mult):
                if broken == "flat-after-increase":
                    return (b * factor + f[t]) * (1 + Fraction(q, 100))
                return (b + f[t]) * mult

            found = []
            if mult > 0:
                centre = math.floor(d / mult - (f[t] if broken != "flat-after-increase" else f[t] / factor))
                width = math.ceil(1 / mult) + 2
                for b in range(centre - width, centre + width + 1):
                    v = display(b)
                    if broken == "display-floored":
                        ok, at_half = d <= v < d + 1, False
                    else:
                        ok, at_half = d - HALF <= v < d + HALF, False
                        if v == d + HALF and broken != "j5-off":
                            ok, at_half = True, True  # J5
                    if ok and (broken != "range-bound" or lo <= b <= hi):
                        found.append((b, at_half))
            detail[t] = {"display": d, "range": (lo, hi), "flat": f[t], "inc": inc, "rolls": []}
            if not found:
                # C93: nothing the item carries gives this display
                return ("undecided", None, f"no roll: no integer gives the {WORD[t]} displayed, {d}", detail,
                        took + ["no-roll"], set())
            if hi == lo:
                detail[t]["rolls"] = [(b, None, h) for b, h in found]
                # J33: a percentile of such a range divides by nothing
                return ("undecided", None, f"range: the base's minimum {WORD[t]} is its maximum", detail,
                        took + ["flat-range"], set())
            took.append("roll")
            per[t] = []
            for b, at_half in found:
                pct = Fraction(100 * (b - lo), hi - lo)
                detail[t]["rolls"].append((b, pct, at_half))
                per[t].append(pct)
                if at_half:
                    took.append("half")
                if b > hi:
                    took.append("roll-over-max")  # C101: reads over 100 %
                elif b < lo:
                    took.append("roll-under-min")  # the rule's letter: any integer
            if t == "ward":
                took.append("ward")
        rounding = (half_even if broken == "average-half-even" else
                    math.floor if broken == "average-floored" else half_up)
        values, at_half = set(), False
        for combo in itertools.product(*per.values()):
            mean = sum(combo) / len(combo)
            at_half |= mean - math.floor(mean) == HALF
            values.add(rounding(mean))
        took.append({1: "one-type", 2: "two-types", 3: "three-types"}.get(len(per), "three-types"))
        if at_half:
            took.append("average-at-a-half")
        if len(values) > 1:
            # J33: the display does not fix the value
            return ("undecided", None, "several values: the display admits rolls that give "
                    + ", ".join(str(v) for v in sorted(values)), detail, took + ["several-values"], values)
        if any(len(v) > 1 for v in per.values()):
            took.append("several-rolls-one-value")
        return "value", min(values), "", detail, took, values

    common = {"taken": taken, "left": left, "quality": quality_text,
              "base_row": f"base-defences.csv row {base_row}", "not_of_base": not_of_base, "lines": lines}

    def said(result):
        status, value, cause, _, _, values = result
        if status == "value":
            return str(value)
        return f"{status}: {cause}"

    # J8, J33: a line whose side cannot be established leaves the field open.
    if opened:
        with_it, without = compute(opened), compute([])
        rule = opened[0]["rule"]
        cause = {"twin-unmarked": "an implicit twin on an item of neither mark",
                 "mark-unread": "an implicit twin on an item whose mark is unread",
                 "twin-both-marks": "an implicit twin on an item both eldritch and synthesised",
                 "twin-other-array": f"a twin text in `{opened[0]['array']}`",
                 "crucible-line": "a defence line in `crucibleMods`"}[rule]
        return answer("undecided", steps, cause=f"{cause}: {opened[0]['row']}", types=without[3],
                      readings=[("were it the item's own", said(with_it)), ("were it left out", said(without))],
                      **common)
    status, value, cause, detail, took, values = compute([])
    return answer(status, steps + took, value=value, cause=cause, types=detail, **common)


# ---- an item from each source, as a private item carries it ---------------------------------------------

def item_key(raw):
    """The same item fetched by two searches differs in its pseudo lines alone (percentile.py)."""
    return json.dumps({k: (dict(v, pseudoMods=None) if k == "lines" else v)
                       for k, v in raw.items() if k not in ("extended", "hashes")}, sort_keys=True)


def captured_facts(raw, marks):
    """A fetched item as a private item carries it: (facts, the lines with their ids, kept apart)."""
    lines, beside = {}, []
    for array, entries in raw["lines"].items():
        if array == "pseudoMods":
            continue
        lines[array] = [(e.get("description"), e.get("flags")) for e in entries]
        beside += [(array, e) for e in entries]
    properties = [(p[1], p[2][0] if p[2] else None) for p in raw.get("properties", [])]
    facts = {"realm": "pc", "base": raw.get("baseType", raw.get("typeLine")), "rarity": raw.get("rarity"),
             "name": raw.get("name") if raw.get("rarity") == "Unique" else "", "marks": marks,
             "properties": properties, "lines": lines}
    return facts, beside


def captured_marks():
    """{locator: {mark: True}} from data/defence-twins.csv, which holds an item's marks where it shows a
    defence and carries a twin line; the first row holding an item."""
    out = {}
    if not TWINS_CSV.exists():
        return out
    with TWINS_CSV.open(newline="\n") as fh:
        next(fh)
        for row in csv.DictReader(fh):
            if row["source"] == "captures":
                out[row["locator"]] = {m: True for m in row["marks"].split(";") if m in MARKS}
    return out


def raw_marks(captures):
    """{case: {mark: True}} from raw/searches, as defence-twins.py reads an item's marks; a case the fetch
    does not hold at its place is left out."""
    out = {}
    if not RAW.is_dir():
        return out
    for name, capture in captures.items():
        items = []
        for fetch in [RAW / f"{name}-fetch.json"] + sorted(
                RAW.glob(f"{name}-fetch-*.json"), key=lambda p: int(p.stem.rsplit("-", 1)[1])):
            if fetch.exists() and fetch.stat().st_size:
                items += [r["item"] for r in json.loads(fetch.read_text())["result"]]
        for index, item in enumerate(capture["fetched"] or []):
            if index < len(items) and items[index].get("baseType") == item.get("baseType"):
                out[f"{name}[{index}]"] = {k: items[index][k] for k in MARKS if k in items[index]}
    return out


def has_implicit_twin(tables, raw):
    return any(template(row) in tables.twins for e in raw["lines"].get("implicitMods", [])
               for row in (e.get("description") or "").split("\n"))


def captured_cases(tables):
    """Every captured case, in the captures' order then the fetch census's:
    (case, source, raw, facts, beside, same_as, site value or None)."""
    out, first, counts = [], {}, Counter()
    marks = captured_marks()
    marks_by_key = {}
    captures = json.loads(CAPTURES.read_text())["searches"] if CAPTURES.exists() else {}
    from_raw = raw_marks(captures)
    counts["searches"] = len(captures)
    counts["marks from raw"] = len(from_raw)
    for case, kept in marks.items():
        if case in from_raw:
            counts["marks in both"] += 1
            counts["marks in both, the same"] += {k for k, v in from_raw[case].items() if v is True} == set(kept)
    for name, capture in captures.items():
        for index, raw in enumerate(capture["fetched"] or []):
            case = f"{name}[{index}]"
            key = item_key(raw)
            same = first.get(key, "")
            if not same:
                first[key] = case
                marks_by_key[key] = marks.get(case)
            # the fetch's own marks; else defence-twins.csv's, held at the first row holding the item; else
            # not kept, which the reading says where it meets an implicit twin
            kept = from_raw.get(case, marks_by_key[key])
            counts["marks not kept"] += kept is None
            facts, beside = captured_facts(raw, kept)
            site = raw.get("extended", {}).get("base_defence_percentile")
            counts["fetched rows"] += 1
            counts["carrying the site's value"] += site is not None
            out.append((case, "captures.json", raw, facts, beside, same, site))
    if FETCH.exists():
        for qid, query in json.loads(FETCH.read_text())["queries"].items():
            for index, raw in enumerate(query["fetched"]):
                counts["fetch-census rows"] += 1
                site = raw.get("extended", {}).get("base_defence_percentile")
                if site is None:
                    counts["fetch-census rows left out"] += 1
                    continue
                case = f"{qid}[{index}]"
                key = item_key(raw)
                same = first.get(key, "")
                if not same:
                    first[key] = case
                # the census keeps no mark
                counts["marks not kept"] += 1
                counts["marks not kept, an implicit twin"] += has_implicit_twin(tables, raw)
                facts, beside = captured_facts(raw, None)
                counts["fetch-census rows read"] += 1
                out.append((case, "fetch-census.json", raw, facts, beside, same, site))
    return out, counts


def copy_items():
    """Every live item of the copy, in an order that holds no id: the facts, and the realm."""
    if not STORE.exists():
        return []
    db = sqlite3.connect(f"file:{STORE}?mode=ro", uri=True)
    out = []
    for realm, rarity, body in db.execute("select realm, rarity, json from items where removed_at is null"):
        body = json.loads(body)
        lines = {}
        for array, entries in body.items():
            if not array.endswith("Mods"):
                continue
            if not isinstance(entries, list):
                lines[array] = entries
                continue
            lines[array] = [(e.get("description"), e.get("flags")) if isinstance(e, dict) else (e, None)
                            for e in entries]
        properties = body.get("properties")
        if isinstance(properties, list):
            properties = [((p.get("name"), (p.get("values") or [[None]])[0][0])
                           if isinstance(p, dict) and (p.get("values") or [[None]])[0] else
                           (p.get("name") if isinstance(p, dict) else None, None)) for p in properties]
        out.append({"realm": realm, "base": body.get("baseType"), "rarity": rarity or "",
                    "name": body.get("name", "") if rarity == "Unique" else "",
                    "marks": {k: body[k] for k in MARKS if k in body},
                    "properties": properties, "lines": lines})
    out.sort(key=sort_key)
    return out


def sort_key(item):
    return (item.get("base") or "", item.get("rarity") or "", item.get("name") or "",
            json.dumps(item["properties"], sort_keys=True, default=str),
            json.dumps(item["lines"], sort_keys=True, default=str),
            json.dumps(item["marks"], sort_keys=True, default=str))


# ---- how a case is written ----------------------------------------------------------------------------

def properties_text(item):
    """The properties the field reads, as displayed."""
    props = item.get("properties")
    if props is None:
        return "(none)"
    if not isinstance(props, list):
        return f"(no array: {json.dumps(props)})"
    out = []
    for element in props:
        name, value = element if isinstance(element, (list, tuple)) and len(element) == 2 else (None, None)
        if not isinstance(name, str):
            out.append(f"(name unread: {json.dumps(name)})")
            continue
        shown_name = D.MARKUP.sub(lambda m: m.group(2) or m.group(1), name)
        if shown_name in PROPERTY or shown_name == "Quality":
            out.append(f"{name} {value if isinstance(value, str) else '(unread: ' + json.dumps(value) + ')'}")
    return "; ".join(out) or "(none the field reads)"


def marks_text(item):
    marks = item.get("marks")
    if marks is None:
        return "(not kept)"
    return ";".join(f"{k}" if v is True else f"{k}={json.dumps(v)}" for k, v in sorted(marks.items())
                    if v is not False) or "-"


def lines_text(tables, item):
    """The lines whose text is a defence stat's, `array[flags]: text`; the others counted."""
    out, others = [], 0
    for array, entries in (item.get("lines") or {}).items():
        if not isinstance(entries, list):
            out.append(f"{array}: (no array: {json.dumps(entries)})")
            continue
        for text, flags in entries:
            if not isinstance(text, str):
                out.append(f"{array}: (no text: {json.dumps(text)})")
                continue
            if not any(template(r) in tables.roles for r in text.split("\n")):
                others += 1
                continue
            unread = {k: v for k, v in (flags or {}).items() if not isinstance(v, bool)}
            mark = ";".join(sorted(k for k, v in (flags or {}).items() if v is True))
            mark += "".join(f";{k}={v!r} (unread)" for k, v in unread.items())
            out.append(f"{array}{'[' + mark.strip(';') + ']' if mark else ''}: " + text.replace("\n", " // "))
    if others:
        out.append(f"(+{others} lines of no defence stat)")
    return " | ".join(out)


def read_text(ans):
    """What the reading read: each type's display, range, flat, increase and rolls; the quality."""
    out = []
    if ans["base_row"]:
        out.append(ans["base_row"])
    for t, d in ans["types"].items():
        rolls = ", ".join(f"{b}" + ("" if pct is None else f" = {plain(pct)}%") + (" (at a half, J5)" if h else "")
                          for b, pct, h in d["rolls"]) or "none"
        out.append(f"{t}: display {d['display']}, range {d['range'][0]}-{d['range'][1]}, flat "
                   f"{plain(d['flat'])}, increase {plain(d['inc'])}%, rolls {rolls}")
    if ans["not_of_base"]:
        out.append("displayed and not of the base: " + "/".join(ans["not_of_base"]))
    if ans["quality"]:
        out.append(f"quality {ans['quality']}")
    return "; ".join(out)


def lines_read_text(ans):
    return " | ".join(f"{line['array']}{'[' + line['flags'] + ']' if line['flags'] else ''}: {line['row']} "
                      f"-> {line['became']} ({line['rule']})" for line in ans["lines"])


def cause_text(ans):
    if ans["status"] == "unruled":
        what, readings = UNRULED[ans["cause"]]
        gives = dict(ans["readings"])
        parts = [f"{name}: {gives.get(name, '')}{' — ' if gives.get(name) else ''}{change}"
                 for name, change in readings]
        return f"{ans['cause']} ({what}) :: " + " || ".join(parts)
    if ans["readings"]:
        return ans["cause"] + " :: " + " || ".join(f"{n}: {g}" for n, g in ans["readings"])
    return ans["cause"]


def rests_on(ans):
    return "; ".join(f"{s}: {'+'.join((STEPS[s][1] if s in STEPS else ['unruled']))}" for s in ans["steps"])


def compare(ans, site):
    """Whether the reading and what the site showed agree; site is its value, or None where it showed none."""
    if ans["status"] in ("undecided", "unruled"):
        return "not comparable"
    if ans["status"] == "lacked":
        return "yes" if site is None else "no"
    return "yes" if site == ans["value"] else "no"


def twin_text(tables, ans, beside):
    """Each twin line: the side the site's id names, beside the side the reading took."""
    if beside is None:
        return ""
    ids = defaultdict(list)
    for array, entry in beside:
        for row in (entry.get("description") or "").split("\n"):
            if template(row) in tables.twins:
                h = entry.get("hash")
                ids[(array, row)].append(tables.site_side.get(h.removeprefix("stat."), "no twin id")
                                         if isinstance(h, str) else "no id")
    out = []
    for line in ans["lines"]:
        if not line["twin"]:
            continue
        sides = ids.get((line["array"], line["row"])) or ["no id"]
        out.append(f"{line['array']}: {line['row']} — site {sides.pop(0)}, reading {line['became']}")
    return " | ".join(out)


HEADER = ["case", "kind", "source", "answers", "same_as", "realm", "base", "rarity", "unique", "marks",
          "properties", "lines", "status", "value", "cause_or_readings", "read", "lines_read", "site",
          "agrees", "twin_site_and_reading", "rests_on", "by_hand", "hand_agrees"]


def row_of(tables, case, kind, source, what, same, item, ans, site, beside, hand=("", "")):
    asked = site if isinstance(site, str) else ("none" if site is None else str(site))
    agrees = "" if isinstance(site, str) else compare(ans, site)
    return [case, kind, source, what, same, item["realm"], item.get("base") or "", item.get("rarity") or "",
            item.get("name") or "", marks_text(item), properties_text(item), lines_text(tables, item),
            ans["status"], "" if ans["value"] is None else ans["value"], cause_text(ans), read_text(ans),
            lines_read_text(ans), asked, agrees, twin_text(tables, ans, beside), rests_on(ans), *hand]


# ---- the composed cases, the ruled cases and the worked ones ---------------------------------------------

def composed(realm, base, rarity, properties, *lines, marks=None, name="", arrays=None):
    out = {}
    for array, text, flags in lines:
        out.setdefault(array, []).append((text, flags))
    out.update(arrays or {})
    return {"realm": realm, "base": base, "rarity": rarity, "name": name, "marks": marks or {},
            "properties": properties, "lines": out}


def line(text, array="explicitMods", flags=None):
    return array, text, flags


def has(step):
    return lambda item, ans: step in ans["steps"]


ABSENT_BASE = "A Base The Range Table Lacks"

# Each ruled case: its name, what it answers, the test an item of the copy must pass (None: composed
# always), and the composed item that stands in where the copy holds none.
RULED = [
    ("r-range", "C101, sentence 1: the item's defence against its base's range — one defence, no line, no "
                "quality",
     lambda i, a: a["status"] == "value" and len(a["types"]) == 1 and not a["lines"] and "quality" not in
     a["steps"] and "half" not in a["steps"] and i["rarity"] == "Normal",
     composed("pc", "Plate Vest", "Normal", [("Armour", "23")])),
    ("r-over-maximum", "C101: an item above its base's current maximum reads over 100 % — the stated "
                       "rule's, no capture's",
     has("roll-over-max"), composed("pc", "Plate Vest", "Normal", [("Armour", "30")])),
    ("r-under-minimum", "J34: a roll under the base's minimum reads below 0, by the rule's letter `any integer`",
     has("roll-under-min"), composed("pc", "Plate Vest", "Normal", [("Armour", "17")])),
    ("r-realm-no-table", "C101: a realm with no table is undecided with that reason, never zero",
     has("realm"), composed("poe2", "Rusted Cuirass", "Normal", [("[Armour]", "45")])),
    ("r-realm-no-table-no-defence", "J35: an item of a realm with no table displaying no defence is undecided",
     has("realm-no-defence"), composed("poe2", "Lazuli Ring", "Normal", [])),
    ("r-average-rounded-half-up", "C101, the stated rule: each roll's percentile, averaged, rounded half up "
                                  "— an average at an exact half",
     has("average-at-a-half"),
     composed("pc", "Scale Doublet", "Normal", [("Armour", "103"), ("Evasion Rating", "109")])),
    ("r-j5-exact-half", "J5: a display at an exact half admits both integers as rolls",
     has("half"), composed("pc", "Cone Helmet", "Normal", [("Quality", "+30%"), ("Armour", "58")])),
    ("r-j6-three-types", "J6, C1: three defence types on one base are the average of three rolls",
     has("three-types"),
     composed("pc", "Sacrificial Garb", "Normal",
              [("Armour", "340"), ("Evasion Rating", "350"), ("Energy Shield", "70")])),
    ("r-j6-enchant", "J6, C7: under the enchant that zeroes quality the quality term is dropped",
     lambda i, a: "enchant" in a["steps"] and a["quality"].endswith("dropped under the enchant")
     and not a["quality"].startswith("0%"),
     composed("pc", "Cone Helmet", "Normal", [("Quality", "+20%"), ("Armour", "44")],
              line("Quality does not increase Defences", "enchantMods"))),
    ("r-j6-ward", "J6, C2: ward is a roll as the others are",
     has("ward"), composed("pc", "Runic Crown", "Normal", [("Ward", "170")])),
    ("r-j6-twin-left-out", "J6, C14: a defence line under its global twin the site leaves out, which the "
                           "item's mark tells",
     has("twin-eldritch"),
     composed("pc", "Plate Vest", "Rare", [("Armour", "23")], line("19% increased Armour", "implicitMods"),
              marks={"searing": True, "tangled": True})),
    ("r-j8-eldritch-implicit", "J8, part 1: an implicit twin on an eldritch item is the global stat and left "
                               "out — `tangled` alone",
     None,
     composed("pc", "Plate Vest", "Normal", [("Armour", "23")], line("19% increased Armour", "implicitMods"),
              marks={"tangled": True})),
    ("r-j8-explicit", "J8, part 2: an explicit twin is the item's own",
     lambda i, a: "twin-explicit" in a["steps"] and a["status"] == "value",
     composed("pc", "Plate Vest", "Magic", [("Armour", "40")], line("+17 to Armour"))),
    ("r-j8-synthesised-implicit", "J8, part 3: an implicit twin on a synthesised item is the item's own",
     has("twin-synthesised"),
     composed("pc", "Plate Vest", "Rare", [("Armour", "40")], line("+17 to Armour", "implicitMods"),
              marks={"synthesised": True})),
    ("r-j8-unmarked-implicit", "J8, part 4: an implicit twin on an item of neither mark is undecided",
     has("twin-unmarked"),
     composed("pc", "Plate Vest", "Rare", [("Armour", "23")], line("19% increased Armour", "implicitMods"))),
    ("r-unmarked-implicit-of-no-roll", "the function's, by rule 8: an implicit twin of a type no roll reads "
                                       "leaves nothing open",
     None,
     composed("pc", "Plate Vest", "Rare", [("Armour", "23")],
              line("+20 to maximum Energy Shield", "implicitMods"))),
    ("r-lacked", "C93, pseudo.rs: an item displaying none of the four defences lacks the field",
     lambda i, a: a["status"] == "lacked" and i["rarity"] == "Rare",
     composed("pc", "Ruby Ring", "Rare", [])),
    ("r-lacked-on-an-armour-base", "the same, of an item whose base the range table holds: it displays no "
                                   "defence, and lacks the field",
     lambda i, a: a["status"] == "lacked" and a["_base_ranged"],
     composed("pc", "Simple Robe", "Unique", [], name="Tabula Rasa")),
    ("r-base-not-in-table", "C93: a base the range table lacks is undecided",
     has("base-lacking"), composed("pc", ABSENT_BASE, "Normal", [("Armour", "100")])),
    ("r-properties-no-array", "pseudo.rs, rule 8: `properties` unread leaves the field undecided",
     None, composed("pc", "Plate Vest", "Normal", "unread")),
    ("r-property-name-unread", "pseudo.rs: an element of `properties` whose name is unread may be one the "
                               "field reads",
     None, composed("pc", "Ruby Ring", "Rare", [(7, "23")])),
    ("r-defence-no-number", "pseudo.rs: a defence displayed as no number",
     None, composed("pc", "Plate Vest", "Normal", [("Armour", "many")])),
    ("r-quality-no-number", "pseudo.rs: the quality displayed as no number, on an item displaying a defence",
     None, composed("pc", "Plate Vest", "Normal", [("Quality", "some"), ("Armour", "23")])),
    ("r-quality-no-number-no-defence", "rule 8, of no more than was lost: the quality unread on an item "
                                       "displaying no defence leaves it lacked",
     None, composed("pc", "Ruby Ring", "Rare", [("Quality", "some")])),
    ("r-several-rolls-different-values", "J33: a display more than one roll gives, the rolls "
                                         "giving different values",
     has("several-values"),
     composed("pc", "Iron Hat", "Magic", [("Armour", "5")], line("50% reduced Armour"))),
    ("r-several-rolls-one-value", "the function's: a display more than one roll gives, every combination "
                                  "rounding to one value",
     has("several-rolls-one-value"),
     composed("pc", "Sacrificial Garb", "Magic",
              [("Armour", "347"), ("Evasion Rating", "350"), ("Energy Shield", "70")],
              line("1% reduced Armour"))),
    ("r-base-held-more-than-once", "the function's: a base name the range table holds more than once, the "
                                   "display telling which",
     has("base-held-twice"),
     composed("pc", "Two-Toned Boots", "Normal", [("Armour", "130"), ("Evasion Rating", "140")])),
    ("r-base-held-more-than-once-open", "the function's: the same name, the display telling none apart",
     has("base-held-twice-open"),
     composed("pc", "Two-Toned Boots", "Rare",
              [("Armour", "130"), ("Evasion Rating", "140"), ("Energy Shield", "28")])),
    ("r-min-is-max", "J33: a base whose minimum is its maximum",
     has("flat-range"), composed("pc", "Kaom's Plate", "Normal", [("Armour", "23")])),
    ("r-defence-base-lacks", "C9: a defence displayed that the base lacks — its own types read",
     lambda i, a: "not-of-base" in a["steps"] and a["status"] == "value",
     composed("pc", "Plate Vest", "Rare", [("Armour", "23"), ("Energy Shield", "20")],
              line("+20 to maximum Energy Shield"))),
    ("r-reduced-line", "C13: a `reduced` local line is its stat below nothing",
     lambda i, a: "reduced" in a["steps"] and a["status"] == "value",
     composed("pc", "Cone Helmet", "Magic", [("Armour", "36")], line("10% reduced Armour"))),
    ("r-base-defence-not-displayed", "J33: the base ranges a defence the item does not display",
     has("base-type-not-shown"), composed("pc", "Scale Doublet", "Normal", [("Armour", "103")])),
    ("r-twin-both-marks", "J33: an implicit twin on an item both eldritch and synthesised",
     has("twin-both-marks"),
     composed("pc", "Plate Vest", "Rare", [("Armour", "23")], line("19% increased Armour", "implicitMods"),
              marks={"tangled": True, "synthesised": True})),
    ("r-twin-other-array", "J33: a twin text in an array J8 does not name",
     has("twin-other-array"),
     composed("pc", "Plate Vest", "Rare", [("Armour", "40")], line("+17 to Armour", "scourgeMods"))),
    ("r-crucible-line", "J33: a defence line in `crucibleMods`",
     has("crucible-line"),
     composed("pc", "Plate Vest", "Rare", [("Armour", "23")], line("19% increased Armour", "crucibleMods"))),
    ("r-array-no-array", "rule 8: an array that is no array is unread",
     None, composed("pc", "Plate Vest", "Rare", [("Armour", "23")], arrays={"explicitMods": "unread"})),
    ("r-line-no-text", "rule 8: a line that is no text is unread",
     None, composed("pc", "Plate Vest", "Rare", [("Armour", "23")], ("explicitMods", None, None))),
    ("r-flag-unread", "J32: a flag that is no yes or no opens only what asks a flag — the line counts by "
                      "its text",
     None,
     composed("pc", "Plate Vest", "Rare", [("Armour", "40")], line("+17 to Armour", flags={"crafted": "yes"}))),
    ("r-mark-unread", "rule 8: a mark that is no yes or no, under an implicit twin",
     None,
     composed("pc", "Plate Vest", "Rare", [("Armour", "23")], line("19% increased Armour", "implicitMods"),
              marks={"searing": "yes"})),
    ("r-rows-of-a-mod", "LEDGER 9c4: a defence line that is one row of a mod displayed over several",
     lambda i, a: "rows-of-a-mod" in a["steps"] and a["status"] == "value",
     composed("pc", "Plate Vest", "Rare", [("Armour", "40")], line("+17 to Armour\n+25 to maximum Life"))),
    ("r-no-roll", "C93: no integer gives the display",
     has("no-roll"), composed("pc", "Cone Helmet", "Normal", [("Quality", "+20%"), ("Armour", "51")])),
    ("r-quality-over-20", "C6: quality above 20 multiplies as any quality does",
     lambda i, a: a["status"] == "value" and "quality" in a["steps"] and int(a["quality"].rstrip("%")) > 20,
     composed("pc", "Cone Helmet", "Normal", [("Quality", "+25%"), ("Armour", "55")])),
    ("r-unique", "C8: a unique is read as any item is, against its base's range",
     lambda i, a: a["status"] == "value" and i["rarity"] == "Unique" and a["taken"],
     composed("pc", "Plate Vest", "Unique", [("Armour", "46")], line("100% increased Armour"), name="(a composed unique)")),
    ("r-local-text-outside-explicit", "the join: a text only a local stat displays is the item's own "
                                      "whatever its array",
     lambda i, a: a["status"] == "value" and any(x["rule"] == "local-only" and x["array"] != "explicitMods"
                                                 for x in a["taken"]),
     composed("pc", "Simple Robe", "Rare", [("Energy Shield", "20")],
              line("10% increased Energy Shield", "implicitMods"))),
    ("r-flagged-line", "J32's ground: a crafted or fractured line counts by its text, no flag asked",
     lambda i, a: a["status"] == "value" and any(x["flags"] for x in a["taken"]),
     composed("pc", "Plate Vest", "Rare", [("Armour", "40")], line("+17 to Armour", flags={"crafted": True}))),
]

# The worked cases: each one's arithmetic written by hand before the function is run on it (the plan,
# rule 3). The ranges in the hand's column were read by eye from ../repoe/data/base-defences.csv; the
# function reads the file, and the script says whether the two agree.
WORKED = [
    ("w-one-type", "one defence type",
     composed("pc", "Cone Helmet", "Normal", [("Armour", "44")]),
     ("value", 50, "Cone Helmet ranges Armour 38-50. No line, no quality: the roll is the display, 44. "
                   "100 x (44 - 38) / (50 - 38) = 600 / 12 = 50.")),
    ("w-two-types", "two defence types, the average rounded half up",
     composed("pc", "Scale Doublet", "Normal", [("Armour", "100"), ("Evasion Rating", "103")]),
     ("value", 19, "Scale Doublet ranges Armour 97-121 and Evasion 97-121. Rolls 100 and 103. Armour: "
                   "100 x 3 / 24 = 12.5. Evasion: 100 x 6 / 24 = 25. (12.5 + 25) / 2 = 18.75, rounded 19.")),
    ("w-flat-increase-quality", "a local flat line and a local increase, with quality",
     composed("pc", "Copper Plate", "Rare", [("Quality", "+20%"), ("Armour", "420")],
              line("+50 to Armour"), line("40% increased Armour")),
     ("value", 53, "Copper Plate ranges Armour 176-221. Display = (roll + 50) x 1.40 x 1.20 = (roll + 50) x "
                   "1.68. Roll 200: 250 x 1.68 = 420 exactly. Roll 199: 418.32, shown 418; roll 201: 421.68, "
                   "shown 422: 200 alone. 100 x (200 - 176) / (221 - 176) = 2400 / 45 = 53.33, rounded 53.")),
    ("w-exact-half", "a display at an exact half (J5)",
     composed("pc", "Cone Helmet", "Normal", [("Quality", "+10%"), ("Armour", "49")]),
     ("value", 58, "Cone Helmet ranges Armour 38-50. Display = roll x 1.10. Roll 44: 48.4, shown 48. Roll 45: "
                   "49.5, which half up shows as 50 — so no roll shows 49 by the stated rule; J5 admits 45 "
                   "at 49 too. Roll 46: 50.6, shown 51. 100 x (45 - 38) / 12 = 58.33, rounded 58.")),
]


# ---- the copy: shapes, and the groups of lines --------------------------------------------------------

SHAPES = [
    ("C1", "three defence types"), ("C2", "ward"), ("C3", "a shield"), ("C4", "a helmet"), ("C5", "gloves"),
    ("C6", "quality above 20"), ("C7", "`Quality does not increase Defences`"), ("C8", "a unique"),
    ("C9", "a defence displayed that the base lacks"), ("C10", "a roll over the base's maximum"),
    ("C12", "a display at an exact half (J5)"), ("C13", "a `reduced` local line"),
    ("C14", "an eldritch implicit twin of a type a roll reads, left out"),
    ("S1", "a synthesised implicit twin, the item's own"),
    ("S2", "a base name the range table holds more than once"),
    ("S3", "a roll under the base's minimum"),
    ("S4", "more than one roll gives a display, one value"),
    ("S5", "a text only a local stat displays, outside `explicitMods`"),
    ("S6", "a line under a flag, taken"),
    ("none", "none of them"),
]
SHAPE_RULE = {"C1": "three-types", "C2": "ward", "C6": "quality", "C7": "enchant", "C9": "not-of-base",
              "C10": "roll-over-max", "C12": "half", "C13": "reduced", "C14": "twin-eldritch",
              "S1": "twin-synthesised", "S2": "base-held-twice", "S3": "roll-under-min",
              "S4": "several-rolls-one-value", "S5": "local-only", "S6": "J32 (flag-unread's ground)",
              "C3": "base", "C4": "base", "C5": "base", "C8": "base", "none": "roll"}


def shapes_of(tables, item, ans):
    s = set()
    steps = ans["steps"]
    names = tables.classes.get(item.get("base"), set())
    for shape, cls in (("C3", "Shields"), ("C4", "Helmets"), ("C5", "Gloves")):
        if cls in names:
            s.add(shape)
    if "three-types" in steps:
        s.add("C1")
    if "ward" in steps:
        s.add("C2")
    if "quality" in steps and int(ans["quality"].rstrip("%")) > 20:
        s.add("C6")
    if "enchant" in steps:
        s.add("C7")
    if item.get("rarity") == "Unique":
        s.add("C8")
    for shape, step in (("C9", "not-of-base"), ("C10", "roll-over-max"), ("C12", "half"), ("C13", "reduced"),
                        ("C14", "twin-eldritch"), ("S1", "twin-synthesised"), ("S2", "base-held-twice"),
                        ("S3", "roll-under-min"), ("S4", "several-rolls-one-value")):
        if step in steps:
            s.add(shape)
    if any(x["rule"] == "local-only" and x["array"] != "explicitMods" for x in ans["taken"]):
        s.add("S5")
    if any(x["flags"] for x in ans["taken"]):
        s.add("S6")
    return s or {"none"}


def group_of(line):
    """A line by template, role, what became of it, array, flags and the item's mark."""
    return (line["template"], line["role"], line["became"], line["rule"], line["array"], line["flags"] or "-",
            line["mark"])


QUESTION = {
    "twin-explicit": "does the site show the reading's value of an item whose explicit `{t}` is the line "
                     "({f}) on an item of the mark `{m}` — the line the item's own?",
    "local-only": "does the site show the reading's value of an item whose `{t}` sits in `{a}` ({f}) on an "
                  "item of the mark `{m}` — the line the item's own?",
    "twin-eldritch": "does the site show the reading's value of an eldritch item whose implicit `{t}` is "
                     "left out?",
    "twin-synthesised": "does the site show the reading's value of a synthesised item whose implicit `{t}` "
                        "is the item's own?",
    "line-of-no-roll": "does the site show the reading's value of an item whose `{t}` in `{a}` ({f}), mark "
                       "`{m}`, is of a type its base ranges none of?",
    "enchant": "does the site show the reading's value of an item whose `{t}` sits in `{a}` on an item of "
               "the mark `{m}` — the quality term dropped?",
}
SHAPE_QUESTION = {
    "C10": "does the site show over 100 of an item whose display no roll of the base's range gives and one "
           "above it does (i03 found none listed)?",
    "S3": "what does the site show of an item whose display only a roll under the base's minimum gives?",
    "S4": "does the site show the reading's value of an item whose display more than one roll gives?",
}


# ---- the run ------------------------------------------------------------------------------------------

def main():
    args = sys.argv[1:]
    if args == ["--breaks"]:
        for name, what in BREAKS.items():
            print(f"{name}: {what}")
        return 0
    only_break = None
    if len(args) == 2 and args[0] == "--break" and args[1] in BREAKS:
        only_break = args[1]
    elif args:
        print(__doc__)
        return 2

    absent = [str(p.relative_to(ROOT)) for p in (CAPTURES, RAW, TWINS_CSV, FETCH, STORE) if not p.exists()]
    print("local inputs absent: " + ("; ".join(absent) if absent else "none")
          + " (the clones are not read by this script)")
    tables = Tables()
    cases, counts = captured_cases(tables)

    def tally(broken=None):
        t = Counter()
        for _, _, _, facts, _, _, site in cases:
            t[compare(read(tables, facts, broken), site)] += 1
        return t

    if only_break:
        base_, t = tally(), tally(only_break)
        print(f"break {only_break} ({BREAKS[only_break]}): {t['no']} of {len(cases)} captured cells disagree, "
              f"{t['not comparable']} not comparable (unbroken: {base_['no']} and {base_['not comparable']}); "
              f"nothing written")
        return 0

    # -- step 2: the reading's tables ---------------------------------------------------------------------
    print("\n== the reading's tables ==")
    held = {n: rs for n, rs in tables.bases.items() if len(rs) > 1}
    flat_ranges = sorted({n for n, rs in tables.bases.items() for _, r in rs for lo, hi in r.values() if lo == hi})
    print(f"realms with a table (classes-v1.toml): {', '.join(sorted(tables.realms))}")
    print(f"base-defences.csv: {sum(len(rs) for rs in tables.bases.values())} rows, {len(tables.bases)} names; "
          f"held more than once: {', '.join(f'{n} ({len(rs)})' for n, rs in sorted(held.items())) or 'none'}; "
          f"a minimum that is its maximum: {', '.join(flat_ranges) or 'none'}; ranging no defence: "
          f"{', '.join(sorted(n for n, rs in tables.bases.items() if any(not r for _, r in rs))) or 'none'}")
    print(f"defence texts (the join): {len(tables.roles)}, of which twin texts {len(tables.twins)}: "
          + "; ".join(sorted(tables.twins)))
    print("texts a local stat alone displays: "
          + "; ".join(sorted(t for t in tables.roles if t not in tables.twins)))

    rows = []
    step_cases = {k: Counter() for k in list(STEPS) + list(UNRULED)}

    def count_steps(kind, ans):
        for s in ans["steps"]:
            step_cases[s][kind] += 1

    # -- step 3, captured ---------------------------------------------------------------------------------
    print("\n== captured ==")
    print(f"data/captures.json: {counts['searches']} searches, {counts['fetched rows']} fetched rows, "
          f"{counts['carrying the site\'s value']} carrying the site's value")
    print(f"fetch-census.json: {counts['fetch-census rows']} fetched rows; read, {counts['fetch-census rows read']} "
          f"— those carrying the site's value, the base read from the type line, the census keeping no "
          f"`baseType`; left out, {counts['fetch-census rows left out']} — the site showed no value of them, "
          f"and with no `baseType` and no mark kept nothing says what a magic one's base is")
    print(f"marks: {counts['marks from raw']} captured rows' from raw/searches; defence-twins.csv holds "
          f"{counts['marks in both']} of those rows, {counts['marks in both, the same']} the same; not kept "
          f"{counts['marks not kept']} (of them carrying an implicit twin text: "
          f"{counts['marks not kept, an implicit twin']})")
    cap = []  # (case, source, facts, ans, site, same, beside)
    for case, source, raw, facts, beside, same, site in cases:
        ans = read(tables, facts)
        cap.append((case, source, facts, ans, site, same, beside))
        count_steps("captured", ans)
        rows.append(row_of(tables, case, "captured", source, "", same, facts, ans, site, beside))
    distinct = [c for c in cap if not c[5]]
    print(f"captured cases: {len(cap)} — read {len(cap)}, left out {counts['fetch-census rows left out']} (above); "
          f"{len(distinct)} distinct items, {len(cap) - len(distinct)} repeats")
    by = Counter((c[3]["status"], "site a value" if c[4] is not None else "site none") for c in cap)
    for (status, site), n in sorted(by.items()):
        print(f"  reading {status}, {site}: {n}")
    by = Counter((c[3]["status"], "site a value" if c[4] is not None else "site none") for c in distinct)
    print("  distinct items: " + "; ".join(f"{s}, {w}: {n}" for (s, w), n in sorted(by.items())))
    valued = [c for c in cap if c[4] is not None]
    valued_distinct = [c for c in valued if not c[5]]
    agree = Counter(compare(c[3], c[4]) for c in cap)
    agree_valued = Counter(compare(c[3], c[4]) for c in valued)
    agree_distinct = Counter(compare(c[3], c[4]) for c in valued_distinct)
    print(f"cells: {len(cap)}; agreeing {agree['yes']}, disagreeing {agree['no']}, not comparable "
          f"{agree['not comparable']}")
    print(f"cells where the site showed a value: {len(valued)}; agreeing {agree_valued['yes']}, disagreeing "
          f"{agree_valued['no']}, not comparable {agree_valued['not comparable']}; distinct items "
          f"{len(valued_distinct)}: agreeing {agree_distinct['yes']}, disagreeing {agree_distinct['no']}, "
          f"not comparable {agree_distinct['not comparable']}")
    print(f"  the README's `of 637 distinct items … 637` (The percentile's round, J5): "
          f"{'agrees' if agree_distinct['yes'] == 637 and len(valued_distinct) == 637 else 'does not agree'} "
          f"— fetch-census.json's {counts['fetch-census rows read']} are counted here and not there: "
          f"captures.json's distinct items carrying the value "
          f"{sum(1 for c in valued_distinct if c[1] == 'captures.json')}, agreeing "
          f"{sum(1 for c in valued_distinct if c[1] == 'captures.json' and compare(c[3], c[4]) == 'yes')}")
    j5 = [c for c in valued_distinct if "half" in c[3]["steps"]]
    print(f"  distinct items answered through J5 (a roll at an exact half): {len(j5)}: "
          + "; ".join(f"{c[0]} {c[2]['base']} site {c[4]} reading {c[3]['value']}" for c in j5))
    # the twins: the side the site's id names, beside the side the reading took
    sides = Counter()
    for c in distinct:
        if not c[3]["lines"]:
            continue
        ids = defaultdict(list)
        for array, entry in c[6]:
            for r in (entry.get("description") or "").split("\n"):
                h = entry.get("hash")
                ids[(array, r)].append(tables.site_side.get(h.removeprefix("stat."), "no twin id")
                                       if isinstance(h, str) else "no id")
        for ln in c[3]["lines"]:
            if ln["twin"]:
                side = (ids.get((ln["array"], ln["row"])) or ["no id"]).pop(0)
                sides[(ln["array"], ln["mark"], side, ln["became"].split(":")[0], ln["rule"])] += 1
    print("twin lines on distinct captured items the reading reached (array, the item's mark, the site's id, "
          "the reading, the step): lines")
    for k, n in sorted(sides.items()):
        print(f"  {' / '.join(k)}: {n}")

    # -- step 3, ruled ------------------------------------------------------------------------------------
    print("\n== ruled ==")
    copy = copy_items()
    print(f"the copy: {len(copy)} live items" + ("" if copy else " — absent: every ruled case is composed"))
    copy_answers = []
    captured_keys = {(c[2]["base"], c[2]["rarity"], properties_text(c[2]), lines_text(tables, c[2])) for c in cap}
    for item in copy:
        ans = read(tables, item)
        ans["_base_ranged"] = any(r for _, r in tables.bases.get(item.get("base"), []))
        copy_answers.append((item, ans))
    ruled = []
    for name, what, test, stand_in in RULED:
        found = None
        if test is not None:
            for item, ans in copy_answers:
                if item["realm"] in tables.realms or name.startswith("r-realm"):
                    if test(item, ans) and (item["base"], item["rarity"], properties_text(item),
                                            lines_text(tables, item)) not in captured_keys:
                        found = item
                        break
        item, source = (found, "the copy") if found else (stand_in, "composed")
        ans = read(tables, item)
        ruled.append((name, what, source, item, ans))
        count_steps("ruled", ans)
        rows.append(row_of(tables, name, "ruled", source, what, "", item, ans, "not asked", None))
    by_source = Counter(r[2] for r in ruled)
    print(f"ruled cases: {len(ruled)} — {by_source['the copy']} items of the copy, {by_source['composed']} composed")
    for name, what, source, item, ans in ruled:
        said = (str(ans["value"]) if ans["status"] == "value" else ans["status"]
                + (f" ({ans['cause']})" if ans["cause"] else ""))
        print(f"  {name} [{source}]: {said}")

    # -- step 3, worked -----------------------------------------------------------------------------------
    print("\n== worked ==")
    worked_ok = 0
    for name, what, item, (status, value, hand) in WORKED:
        ans = read(tables, item)
        same = ans["status"] == status and ans["value"] == value
        worked_ok += same
        count_steps("worked", ans)
        rows.append(row_of(tables, name, "worked", "composed", what, "", item, ans, "not asked", None,
                           hand=(f"{hand} Answer: {value}.", "yes" if same else "no")))
        print(f"  {name}: by hand {value}, the function {ans['value'] if ans['status'] == 'value' else ans['status']}"
              f" — {'the same' if same else 'NOT the same'}; read: {read_text(ans)}")
    print(f"worked cases: {len(WORKED)}; the function gave what the hand did on {worked_ok} of {len(WORKED)}")

    with OUT.open("w", newline="") as fh:
        fh.write(f"# generated by scripts/percentile-expected.py from {INPUTS}; a row a case: what "
                 f"`pseudo.defence_pct` answers of the item, what was read, and what the site showed\n")
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(HEADER)
        w.writerows(rows)

    print("\n== the steps of the function: cases that take each, captured / ruled / worked ==")
    none_take = []
    for step in list(STEPS) + list(UNRULED):
        c = step_cases[step]
        cites = "+".join(STEPS[step][1]) if step in STEPS else "unruled"
        print(f"  {step}: {c['captured']} / {c['ruled']} / {c['worked']}   [{cites}]")
        if not sum(c.values()):
            none_take.append(step)
    print("steps no case takes: " + (", ".join(none_take) or "none"))
    print(f"steps: {len(STEPS)} cited, {len(UNRULED)} unruled; cases: captured {len(cap)}, ruled {len(ruled)}, "
          f"worked {len(WORKED)}")

    # -- the unruled cells --------------------------------------------------------------------------------
    print("\n== unruled ==")
    for key, (what, readings) in UNRULED.items():
        cells = [(n, k) for n, k, a in ([(c[0], "captured", c[3]) for c in cap]
                                        + [(r[0], "ruled", r[4]) for r in ruled]) if a["status"] == "unruled"
                 and a["cause"] == key]
        on_copy = sum(1 for _, a in copy_answers if a["status"] == "unruled" and a["cause"] == key)
        print(f"  {key}: {what}")
        print(f"    cells: captured {sum(1 for _, k in cells if k == 'captured')}, ruled "
              f"{', '.join(n for n, k in cells if k == 'ruled') or 'none'}; live items of the copy: {on_copy}")
        for name, change in readings:
            print(f"    - {name}: {change}")

    # -- step 4: disagreements, and what the agreement is worth ---------------------------------------
    print("\n== disagreements ==")
    wrong = [c for c in cap if compare(c[3], c[4]) == "no"]
    print(f"captured cells where the reading and the site differ: {len(wrong)} of {len(cap)}")
    for c in wrong:
        print(f"  {c[0]} {c[2]['base']}: site {c[4]}, reading "
              f"{c[3]['value'] if c[3]['status'] == 'value' else c[3]['status']}; {read_text(c[3])}")
    open_cells = [c for c in cap if compare(c[3], c[4]) == "not comparable"]
    print(f"captured cells not comparable (the reading undecided or unruled): {len(open_cells)}")
    for c in open_cells:
        print(f"  {c[0]} {c[2]['base']}: site {'none' if c[4] is None else c[4]}, reading {c[3]['status']} — "
              f"{cause_text(c[3])}")
    print("\n== broken once for each rule: captured cells that then disagree / not comparable, of "
          f"{len(cap)} ==")
    unbroken = tally()
    alone = []
    for name, what in BREAKS.items():
        t = tally(name)
        noticed = (t["no"], t["not comparable"]) != (unbroken["no"], unbroken["not comparable"])
        print(f"  {name}: {t['no']} / {t['not comparable']} — {what}")
        if not noticed:
            alone.append(name)
    print(f"breaks: {len(BREAKS)}; a break no capture notices rests on the ruling alone: {len(alone)}: "
          + ", ".join(alone))

    # -- step 5: what no capture has asked ----------------------------------------------------------------
    print("\n== what no capture has asked ==")
    live_pc = [(i, a) for i, a in copy_answers if i["realm"] in tables.realms]
    to_value = [(i, a) for i, a in live_pc if a["status"] == "value"]
    answered = [c for c in cap if c[4] is not None and compare(c[3], c[4]) == "yes"]
    answered_distinct = [c for c in answered if not c[5]]
    print(f"the copy's live pc items: {len(live_pc)}; read to a value: {len(to_value)}; captured cells the site "
          f"answered and the reading agrees with: {len(answered)} ({len(answered_distinct)} distinct items)")
    groups = defaultdict(lambda: Counter())
    for i, a in to_value:
        seen = set()
        for ln in a["lines"]:
            g = group_of(ln)
            groups[g]["lines"] += 1
            if g not in seen:
                groups[g]["items"] += 1
                seen.add(g)
    for c in answered:
        seen = set()
        for ln in c[3]["lines"]:
            g = group_of(ln)
            if g in groups:
                groups[g]["captured lines"] += 1
                if g not in seen:
                    groups[g]["captured cells"] += 1
                    seen.add(g)
    unasked = {g: n for g, n in groups.items() if not n["captured cells"]}
    print(f"groups of lines (template, role, what became of it, array, flags, the item's mark): {len(groups)}, "
          f"holding {sum(n['lines'] for n in groups.values())} lines; no capture asked {len(unasked)}, holding "
          f"{sum(n['lines'] for n in unasked.values())} lines on "
          f"{sum(n['items'] for n in unasked.values())} item-groups")
    by_rule = defaultdict(lambda: Counter())
    for g, n in groups.items():
        r = by_rule[g[3]]
        r["groups"] += 1
        r["lines"] += n["lines"]
        if not n["captured cells"]:
            r["groups not asked"] += 1
            r["lines not asked"] += n["lines"]
    print("by the rule a line rests on — groups, lines; not asked: groups, lines:")
    for rule, r in sorted(by_rule.items()):
        print(f"  {rule}: {r['groups']}, {r['lines']}; not asked {r['groups not asked']}, {r['lines not asked']}")
    print("not asked, by rule, array, flags and the item's mark — groups, lines:")
    fold = defaultdict(lambda: Counter())
    for g, n in unasked.items():
        f = fold[(g[3], g[4], g[5], g[6])]
        f["groups"] += 1
        f["lines"] += n["lines"]
    for k, f in sorted(fold.items()):
        print(f"  {' / '.join(k)}: {f['groups']}, {f['lines']}")
    print("not asked, by rule and what no capture holds of it — groups, lines:")
    fold = defaultdict(lambda: Counter())
    for g, n in unasked.items():
        what = (f"under the flag `{g[5]}`" if g[5] != "-" else f"on an item of the mark `{g[6]}`")
        f = fold[(g[3], what)]
        f["groups"] += 1
        f["lines"] += n["lines"]
    for k, f in sorted(fold.items()):
        print(f"  {' / '.join(k)}: {f['groups']}, {f['lines']}")
    shape_copy, shape_cap = Counter(), Counter()
    for i, a in to_value:
        for s in shapes_of(tables, i, a):
            shape_copy[s] += 1
    for c in answered_distinct:
        for s in shapes_of(tables, c[2], c[3]):
            shape_cap[s] += 1
    print("items by shape — the copy's read to a value; distinct captured items the site answered:")
    for s, what in SHAPES:
        asked = "" if shape_cap[s] or not shape_copy[s] else "   <- not asked"
        print(f"  {s} ({what}): {shape_copy[s]}; {shape_cap[s]}{asked}")
    shapes_unasked = [s for s, _ in SHAPES if shape_copy[s] and not shape_cap[s]]
    print("shapes the copy holds and no capture asked: " + (", ".join(shapes_unasked) or "none"))
    ruled_only = [s for s in STEPS if step_cases[s]["captured"] == 0]
    print(f"steps no captured case takes (each rests on its ruled case): {len(ruled_only)} of {len(STEPS)}: "
          + ", ".join(ruled_only) + f"; and the {len(UNRULED)} unruled")

    # -- step 6: the copy ---------------------------------------------------------------------------------
    print("\n== the copy ==")
    showing = sum(1 for i, a in copy_answers if displayed(i.get("properties"))[0])
    print(f"live items: {len(copy_answers)}; displaying a defence: {showing} (percentile-shapes.py: 2272 — "
          f"{'the same' if showing == 2272 else 'not the same'})")
    status = Counter()
    for i, a in copy_answers:
        cause = a["cause"].split(":")[0] if a["status"] in ("undecided", "unruled") else ""
        status[(a["status"], cause)] += 1
    for (s, cause), n in sorted(status.items()):
        print(f"  {s}{' — ' + cause if cause else ''}: {n}")
    armour = [(i, a) for i, a in copy_answers if displayed(i.get("properties"))[0]]
    by = Counter((a["status"], a["cause"].split(":")[0] if a["status"] != "value" else "") for i, a in armour)
    print("the items displaying a defence, by reading: "
          + "; ".join(f"{s}{' — ' + c if c else ''} {n}" for (s, c), n in sorted(by.items())))
    half = [(i, a) for i, a in armour if "half" in a["steps"]]

    def float_none(a):
        """Whether percentile.py's float arithmetic finds no roll in range for one of the types read."""
        q = 0 if not a["quality"][0].isdigit() or "dropped" in a["quality"] else int(a["quality"].rstrip("%"))
        return any(not P.rolls(d["display"], q, float(d["inc"]), float(d["flat"]), *d["range"])
                   for d in a["types"].values())

    lost = [(i, a) for i, a in armour if a["status"] == "value" and "twin-eldritch" not in a["steps"]
            and float_none(a)]
    print(f"what became of those percentile-shapes.py left unread: its 22 `realm poe2` — "
          f"{sum(1 for i, a in armour if 'realm' in a['steps'])} undecided, the realm; its 9 at an exact half "
          f"(`none in range`, by percentile.py's floats) — {len(lost)} items here, each to a value: "
          f"{sum(1 for i, a in lost if 'half' in a['steps'])} through J5, and "
          f"{sum(1 for i, a in lost if 'half' not in a['steps'])} by the stated rule alone, their display a "
          f"product at an exact half that rounds up and that a float puts under it "
          f"({', '.join(sorted(i['base'] for i, a in lost if 'half' not in a['steps']))}); its 1 `under the "
          f"minimum` (C14's trace) — eldritch implicit twins of a type a roll reads left out: "
          f"{sum(1 for i, a in armour if 'twin-eldritch' in a['steps'])} item, "
          f"{sum(1 for i, a in armour if 'twin-eldritch' in a['steps'] and a['status'] == 'value')} to a value")
    print(f"items of the copy answered through J5: {len(half)}, of which percentile.py's floats read "
          f"{sum(1 for i, a in half if not float_none(a))} by the stated rule")
    cap_float = [c for c in valued_distinct if c[3]["status"] == "value" and float_none(c[3])]
    print(f"the same over the distinct captured items the site answered: float finds no roll of "
          f"{len(cap_float)}, {sum(1 for c in cap_float if 'half' in c[3]['steps'])} of them through J5 here; "
          f"through J5 here {len(j5)}, of which the floats read "
          f"{sum(1 for c in j5 if not float_none(c[3]))} by the stated rule")
    for i, a in armour:
        if (a["status"] in ("undecided", "unruled") and "realm" not in a["steps"]
                and "realm-no-defence" not in a["steps"]):
            print(f"  {a['status']}: {i['rarity']} {i['base']}"
                  f"{' (' + i['name'] + ')' if i['rarity'] == 'Unique' else ''} — {cause_text(a)}; "
                  f"{properties_text(i)}; {lines_text(tables, i)}")
    over = [(i, a) for i, a in armour if a["status"] == "value" and not 0 <= a["value"] <= 100]
    print(f"values outside 0 to 100 on the copy: {len(over)}")
    unread_elements = sum(1 for i, a in copy_answers for arr in (i["lines"] or {}).values()
                          if isinstance(arr, list) for t, _ in arr if not isinstance(t, str))
    print(f"elements of a lines array that are no text, on the copy: {unread_elements}; on an item displaying "
          f"a defence: {sum(1 for i, a in armour for arr in i['lines'].values() if isinstance(arr, list) for t, _ in arr if not isinstance(t, str))}")

    with OUT_COPY.open("w", newline="") as fh:
        fh.write(f"# generated by scripts/percentile-expected.py from {INPUTS}; the function over every live "
                 f"item of the copy: `class` rows, the answers by class, status and cause; `lines` rows, each "
                 f"defence line of the live pc items read to a value, by group, beside the captured cells of "
                 f"the same group the site answered; `shape` rows, those items by shape\n")
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(["section", "class_or_template_or_shape", "status_or_role", "cause_or_became", "rule", "array",
                    "flags", "mark", "items", "lines", "captured_cells", "captured_lines", "asked",
                    "a_sitting_would_ask"])
        by_class = Counter()
        for i, a in copy_answers:
            cls = tables.class_of(i.get("base")) if i["realm"] in tables.realms else f"({i['realm']}: no table)"
            cause = a["cause"].split(":")[0] if a["status"] in ("undecided", "unruled") else ""
            by_class[(cls, a["status"], cause)] += 1
        for (cls, s, cause), n in sorted(by_class.items()):
            w.writerow(["class", cls, s, cause, "", "", "", "", n, "", "", "", "", ""])
        for g, n in sorted(groups.items()):
            t, role, became, rule, array, flags, mark = g
            asked = "yes" if n["captured cells"] else "no"
            q = "" if n["captured cells"] else QUESTION[rule].format(t=t, a=array, f=flags, m=mark)
            w.writerow(["lines", t, role, became, rule, array, flags, mark, n["items"], n["lines"],
                        n["captured cells"], n["captured lines"], asked, q])
        for s, what in SHAPES:
            asked = "yes" if shape_cap[s] else ("no" if shape_copy[s] else "none on the copy")
            w.writerow(["shape", f"{s}: {what}", "value", "", SHAPE_RULE[s], "", "", "", shape_copy[s], "",
                        shape_cap[s], "", asked, SHAPE_QUESTION.get(s, "") if asked != "yes" else ""])
    print(f"\nwrote {OUT.relative_to(ROOT)} ({len(rows)} rows) and {OUT_COPY.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
