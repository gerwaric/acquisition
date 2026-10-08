#!/usr/bin/env python3
"""The ranged family's expected answers (the build plan, 9d-C; BRIEF.md):
writes ../data/ranged-expected.csv and ../data/ranged-expected-copy.csv and
prints every count the track's README states of them.

    ranged-expected.py                 # write both files, print the counts
    ranged-expected.py --break NAME    # the reading with one rule broken: print how many
                                       # captured cells then disagree, write nothing
    ranged-expected.py --breaks        # list the breaks

One function, `read`, goes from what a private item carries — its realm, its
base, its rarity, each line's text, array and flags — to what each of the 21
totals named `adds_…` answers of it: `lacked`, a value (`low`, `high`, `avg`),
or `undecided` with its cause; `unruled` is kept for a cell the rulings give
two readings of, and no cell is one. It never reads a line's `hash` or `mods`: the
site's id is read beside it (`site_twin`), to say which twin the site filed a
captured line under, and nothing in `read` takes it.

The rows are data/table-changes.csv's (`add row` and `twin` of the 21 totals),
read here and never typed. What the item is comes from the shipped class and
category tables (crates/acquisition-search/reference/classes-v1.toml,
categories-v1.toml): a weapon is an item whose class's id is `weapon` or under
it by the dot (category.rs, J2).

Three rulings of 2026-10-08 are in the reading, each cited by the cells it
decides (CITES: `left-out`, `pair`, `cancels`): a total leaves out the arrays
`crucibleMods` and `utilityMods`, and no other array; where both twins of the
unsuffixed line count toward a total at one weight the row is the line whatever
the item is, as the table of changes now holds the seven attacks' rows; a
ranged total is nothing only where its low and its high are both nothing.

Not carried from class.rs, since no item of the family needs them: a blighted
map's prefix, Growing Wombgift, a captured beast. The script prints how many
live items carrying a line a row names have a base the table lacks.

Inputs: data/table-changes.csv, data/captures.json, data/ranged-census.csv,
data/ranged-uniques.csv, the three shipped tables, ../repoe/data/
class-to-trade-category.csv, ../trade-query/data/stats-2026-09-12.json (the
`(Local)` ids, beside the reading), and the owner's store copy, read-only
(../item-facts/raw/m3/store/mock/GERWARIC_7694.db, `mode=ro`). A local input
that is absent is named and the script goes on with what is there.

Numbers are exact: a displayed number is a fraction, never a float.
"""

import csv
import json
import re
import sqlite3
import sys
import tomllib
from collections import Counter, defaultdict
from fractions import Fraction
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
SEARCH = TRACK.parent
ROOT = SEARCH.parent
CHANGES = TRACK / "data" / "table-changes.csv"
CAPTURES = TRACK / "data" / "captures.json"
CENSUS = TRACK / "data" / "ranged-census.csv"
UNIQUES = TRACK / "data" / "ranged-uniques.csv"
CLASSES = ROOT / "crates/acquisition-search/reference/classes-v1.toml"
CATEGORIES = ROOT / "crates/acquisition-search/reference/categories-v1.toml"
TOTALS = ROOT / "crates/acquisition-search/reference/totals-v6.toml"
TRADE_CLASSES = SEARCH / "repoe/data/class-to-trade-category.csv"
TRADE_STATS = SEARCH / "trade-query/data/stats-2026-09-12.json"
STORE = SEARCH / "item-facts/raw/m3/store/mock/GERWARIC_7694.db"
OUT = TRACK / "data" / "ranged-expected.csv"
OUT_COPY = TRACK / "data" / "ranged-expected-copy.csv"
INPUTS = ("data/table-changes.csv, data/captures.json, data/ranged-uniques.csv, "
          "crates/acquisition-search/reference/classes-v1.toml, categories-v1.toml and totals-v6.toml, "
          "../trade-query/data/stats-2026-09-12.json, "
          "../item-facts/raw/m3/store/mock/GERWARIC_7694.db (read-only)")

NUM = re.compile(r"(?<![\d#])[-+]?\d+(?:\.\d+)?")  # scripts/ranged-census.py, template()
MARKUP = re.compile(r"\[([^\[\]|]*)(?:\|([^\[\]]*))?\]")  # poe2's markup, for the census check alone
EQUIPMENT = {"Normal", "Magic", "Rare", "Unique"}

# The sentences a step of the reading carries out, cited by key in `rests_on`.
CITES = {
    "realm": "totals.rs, C94: no definition for the item's realm is never a zero — unavailable, undecided "
             "with that reason (the table's realms: totals-v6.toml)",
    "category": "DESIGN.md, C94 — the ranged family: `Which it is the item's category decides: on a weapon "
                "the line is the weapon's own` (J7, LEDGER 9d); category.rs, J2: `weapon` and every id under it",
    "own": "DESIGN.md, C94 — the ranged family: `a weapon's own line feeds the attacks' total alone`",
    "other": "DESIGN.md, C94 — the ranged family: `the other twin the plain, the attacks' and the spells'`",
    "row": "totals.rs, C94: a row names the contributing line, its template exact, any source and any flag; "
           "the row is data/table-changes.csv's",
    "range": "decisions/search.md, C94: a total over ranged lines is a range, summed slot-wise, `low`, `high` "
             "and `avg`; table-changes.csv, rule: the average of a line's two numbers; DESIGN.md: bare is `avg`",
    "nothing": "totals.rs, C94: a total of nothing is lacked — no line a row names, or lines that cancel; "
               "table-changes.csv, rule: absent, never 0 (LEDGER 9c)",
    "unread": "totals.rs, C94: a contributor the search could not read — a source a row admits unread, an "
              "occurrence whose number is unread — leaves a subtotal incomplete; the plan, rule 8",
    "unread-flag": "LEDGER 9d, J32: a flag that is no yes or no leaves open only what asks a flag, and no row "
                   "of the table does: the line counts by its text (the plan, rule 8; C93)",
    "open": "decisions/search.md, C93: what cannot be established makes only the terms that needed it "
            "undecided; class.rs: a base the table cannot class",
    "frame": "class.rs: a base under several classes is read by the item's frame, each class that remains; "
             "category.rs, J17: an id every candidate class gives holds",
    "rows-of-a-mod": "LEDGER 9c4: a mod displayed over several rows is named by a row of it and reads that "
                     "row's number",
    "left-out": "ruling 1, 2026-10-08; table-changes.csv, rule: a total leaves out a crucible node's lines and "
                "a flask's own lines, as the site does (n01–n09) — the arrays `crucibleMods` and `utilityMods`, "
                "and no other array. The owner, of both: \"Leave both out.\"",
    "pair": "ruling 2, 2026-10-08, the session's recommendation, not objected to: where both twins of the "
            "unsuffixed line count toward a total at one weight, the total's row is the line whatever the item "
            "is — an item whose side cannot be established answers its attacks' totals, and is undecided "
            "under the plain and the spells'",
    "cancels": "ruling 3, 2026-10-08, the session's recommendation, not objected to: a ranged total is nothing "
               "only where its low and its high are both nothing, as pseudo.rs reads it — a range that cancels "
               "in its average alone is a value",
    "limit": "DESIGN.md, C94 — the ranged family: every ranged total prints the limit — a global line on a "
             "weapon is read as the weapon's own (round i, i06–i08)",
}

# Where the rulings give two readings: the cell is `unruled` and carries both. None is: the two
# questions held here until 2026-10-08, `side-open-attacks` and `range-cancels`, are rulings 2 and 3.
UNRULED = {}

# Ruling 1: the arrays no total reads, each beside the break that counts it.
LEFT_OUT = {"crucibleMods": "crucible-counted", "utilityMods": "utility-counted"}

# One break for each rule of the reading (step 4): the rule, and what the break does.
BREAKS = {
    "realm": "a realm with no recipe is read by pc's rows",
    "weapon-own-as-other": "a weapon's unsuffixed line is read as the other twin",
    "other-as-weapon-own": "an unsuffixed line on an item that is no weapon is read as a weapon's own",
    "side-open-as-other": "an item whose side is open is read as no weapon",
    "frame-picks-none": "a base under classes of both kinds is left open whatever its rarity",
    "low-for-avg": "the low number is read where the average is",
    "nothing-is-zero": "a total of nothing is 0, not lacked",
    "first-line-only": "a total counts the first line a row names and no other",
    "explicit-only": "a line outside explicitMods is not read",
    "unflagged-only": "a line under a flag (crafted, fractured) is not read",
    "whole-text": "a mod displayed over several rows is read by its whole text, not by a row",
    "unread-ignored": "an unread array is read as absent",
    "no-local-row": "a weapon's own line feeds nothing: no row counts the unsuffixed line on a weapon (the "
                    "table's `local` rows until ruling 2, since then the attacks' rows of `any` twin)",
    "no-eldritch-rows": "the table's rows of the two eldritch forms are dropped",
    "no-spells-and-attacks-rows": "the table's `to Spells and Attacks` rows are dropped",
    "no-aggregate-rows": "the rows of the elemental and the all-damage totals are dropped",
    "crucible-counted": "a crucible node's line is counted: `crucibleMods` is read as any other array",
    "utility-counted": "a flask's own line is counted: `utilityMods` is read as any other array",
    "pair-asks-side": "an attacks' row of the unsuffixed line asks the side: open, the total is undecided",
    "cancels-is-lacked": "a range that cancels in its average alone is lacked",
}


def plain(value):
    """A fraction as the site would print it: exact, a decimal where one exists."""
    if value is None:
        return ""
    if value.denominator == 1:
        return str(value.numerator)
    scaled = value * 1000
    if scaled.denominator == 1:
        sign = "-" if value < 0 else ""
        whole, frac = divmod(abs(scaled.numerator), 1000)
        return f"{sign}{whole}.{frac:03d}".rstrip("0")
    return f"{value.numerator}/{value.denominator}"


def template(text):
    return NUM.sub("#", text).replace("+#", "#")


def numbers(text):
    return [Fraction(n) for n in NUM.findall(text)]


class Tables:
    """The rows, the classes and what a weapon is, each read from its file."""

    def __init__(self):
        with CHANGES.open(newline="\n") as fh:
            changes = list(csv.DictReader(fh))
        family = [r for r in changes if r["total"].startswith("adds_")]
        self.totals = {}   # name -> (site id, site text), in the file's order
        self.rows = defaultdict(list)  # name -> [(template, twin, weight)]
        for r in family:
            if r["change"] == "new total":
                self.totals[r["total"]] = (r["site_id"], r["site_text"])
            elif r["change"] in ("add row", "twin"):
                self.rows[r["total"]].append((r["template"], r["twin_or_reads"], Fraction(r["weight"])))
        self.family_changes = Counter(r["change"] for r in family)
        self.family_twins = Counter(r["twin_or_reads"] for r in family if r["change"] == "twin")
        self.rules = [r["template"] for r in changes if r["change"] == "rule"]
        self.templates = {t for rows in self.rows.values() for (t, _, _) in rows}
        self.twinned = {t for rows in self.rows.values() for (t, which, _) in rows if which != "any"}
        self.by_site = {site: name for name, (site, _) in self.totals.items()}

        self.realms = set(tomllib.loads(TOTALS.read_text())["realms"])
        classes = tomllib.loads(CLASSES.read_text())["class"]
        self.class_names = [c["name"] for c in classes]
        self.classes = defaultdict(set)
        for c in classes:
            for b in c["bases"]:
                self.classes[b].add(c["name"])
        ids = tomllib.loads(CATEGORIES.read_text())["id"]
        self.leaf = {}       # class -> its own id
        self.name_rules = defaultdict(list)  # class -> [(rule, id)]
        for i in ids:
            for cls in i.get("classes", []):
                self.leaf[cls] = i["id"]
            if "rule" in i:
                self.name_rules[i["rule"]["class"]].append((i["rule"], i["id"]))

    def id_of(self, cls, base):
        """The id an item of this class has of its own (category.rs: a leaf is a
        class's, or a class's with a rule on the base name)."""
        for rule, leaf in self.name_rules.get(cls, []):
            if ("starts" in rule and base.startswith(rule["starts"])
                    or "ends" in rule and base.endswith(rule["ends"])
                    or "contains" in rule and rule["contains"] in base):
                return leaf
        return self.leaf.get(cls, "")

    def class_is_weapon(self, cls, base=""):
        leaf = self.id_of(cls, base)
        return leaf == "weapon" or leaf.startswith("weapon.")

    def side(self, item, broken=None):
        """('weapon' | 'not a weapon' | None, the cause where None, the cites)."""
        base = item.get("base")
        if not base:
            return None, "the item has no base", ["open"]
        names = self.classes.get(base)
        if not names:
            return None, "the base is not in the class table", ["open"]
        cites = ["category"]
        if len(names) > 1:
            # class.rs: the frame picks among candidates — a gem frame the gem
            # classes, any other the rest. Rarity is what this reading is given:
            # an equipment rarity is no gem frame.
            cites.append("frame")
            if item.get("rarity") in EQUIPMENT and broken != "frame-picks-none":
                names = {n for n in names if "Gems" not in n} or names
        kinds = {self.class_is_weapon(n, base) for n in names}
        if kinds == {True}:
            return "weapon", "", cites
        if kinds == {False}:
            return "not a weapon", "", cites
        return None, "the base is under classes of both kinds and the rarity picks none", cites + ["open"]


def read(tables, item, broken=None):
    """What each of the 21 totals answers of an item.

    item: {"realm", "base", "rarity", "lines": {array: [(text, flags)] or a
    thing that is no list}}. Returns {total: answer}; an answer is
    {"status": lacked|value|undecided|unruled, "low", "high", "avg", "cause",
    "counted": [(line index, array, flags, row text, row, twin taken)],
    "cites": [...], "marks": [...]}.
    """
    answers = {}
    # totals.rs, C94: no definition for the item's realm is never a zero.
    if item["realm"] not in tables.realms and broken != "realm":
        for total in tables.totals:
            answers[total] = {"status": "undecided", "cause": f"realm: `{item['realm']}` has no recipe",
                              "counted": [], "cites": ["realm"], "marks": [], "low": None, "high": None,
                              "avg": None}
        return answers

    # DESIGN.md, C94 — the ranged family; J7: which twin a line is, the item's category decides.
    side, side_cause, side_cites = tables.side(item, broken)
    if side is None and broken == "side-open-as-other":
        side = "not a weapon"
    twin = {"weapon": "local", "not a weapon": "global", None: None}[side]
    if broken == "weapon-own-as-other" and twin == "local":
        twin = "global"
    elif broken == "other-as-weapon-own" and twin == "global":
        twin = "local"

    # The plan, rule 8: what could not be read is unread at its grain — the array, the flag.
    unread_arrays = []
    occurrences = []  # (index, array, flags, row text, template, numbers, flag unread, of several rows)
    left_out = set()  # the templates of lines in an array no total reads
    index = 0
    for array, entries in item["lines"].items():
        # Ruling 1: a total leaves out a crucible node's lines and a flask's own lines — by the array,
        # read or unread, and no other array.
        if array in LEFT_OUT and broken != LEFT_OUT[array]:
            if isinstance(entries, list):
                index += len(entries)
                left_out.update(template(row) for text, _ in entries if isinstance(text, str)
                                for row in text.split("\n"))
            continue
        if not isinstance(entries, list):
            if broken != "unread-ignored":
                unread_arrays.append(array)
            continue
        for text, flags in entries:
            here = index
            index += 1
            flag_unread = any(not isinstance(v, bool) for v in (flags or {}).values())
            shown = ";".join(sorted(k for k, v in (flags or {}).items() if v is True))
            if broken == "explicit-only" and array != "explicitMods":
                continue
            if broken == "unflagged-only" and shown:
                continue
            # LEDGER 9c4: a mod displayed over several rows is named by a row of it.
            rows = text.split("\n") if broken != "whole-text" else [text]
            for row in rows:
                occurrences.append((here, array, shown, row, template(row), numbers(row), flag_unread,
                                    len(rows) > 1))

    for total, (_, _) in tables.totals.items():
        rows = tables.rows[total]
        if broken == "no-eldritch-rows":
            rows = [r for r in rows if not r[0].startswith("While a ")]
        if broken == "no-spells-and-attacks-rows":
            rows = [r for r in rows if not r[0].endswith(" to Spells and Attacks")]
        if broken == "no-aggregate-rows" and re.match(r"adds_(elemental|damage)", total):
            rows = []
        by_template = defaultdict(dict)
        for t, which, weight in rows:
            by_template[t][which] = weight
        low = high = Fraction(0)
        counted, cites, marks = [], set(), set()
        open_side = False      # a line whose row this total cannot name without the side
        flag_open = False
        if left_out & set(by_template):
            cites.add("left-out")
        for (i, array, flags, row, t, nums, flag_unread, several) in occurrences:
            whiches = by_template.get(t)
            if not whiches:
                continue  # totals.rs: a readable line absent from every row is not a contributor
            if broken == "no-local-row" and twin == "local" and t in tables.twinned:
                continue
            if "any" in whiches:
                which, weight = "any", whiches["any"]
                if twin is None and t in tables.twinned:
                    # Ruling 2: the pair is one row, the line whatever the item is.
                    cites.add("pair")
                    cites.update(side_cites)
                    if broken == "pair-asks-side":
                        open_side = True
                        continue
            elif twin is not None:
                if twin not in whiches:
                    cites.add("own" if twin == "local" else "other")
                    cites.update(side_cites)
                    continue  # the row means the other twin: this total does not count the line
                which, weight = twin, whiches[twin]
                cites.add("own" if twin == "local" else "other")
                cites.update(side_cites)
            else:
                cites.update(side_cites)
                if (set(whiches) == {"local", "global"} and whiches["local"] == whiches["global"]
                        and broken != "pair-asks-side"):
                    which, weight = "local or global", whiches["local"]  # ruling 2, a table that holds the pair
                    cites.add("pair")
                else:
                    open_side = True
                    continue
            if flag_unread:
                # J32: no row asks a flag, so a flag that is no yes or no loses nothing a total reads
                cites.add("unread-flag")
            if len(nums) < 2:
                flag_open = True
                continue
            low += nums[0] * weight
            high += nums[1] * weight
            cites.add("row")
            if several:
                cites.add("rows-of-a-mod")
                marks.add("line break")
            counted.append((i, array, flags, row, f"{total} · {t} · {which}", which))
            if broken == "first-line-only":
                break
        answer = {"counted": counted, "marks": sorted(marks), "low": None, "high": None, "avg": None,
                  "cause": ""}
        if unread_arrays:
            answer.update(status="undecided", cause="unread: " + ", ".join(unread_arrays) + " is no array")
            cites.add("unread")
        elif flag_open:
            answer.update(status="undecided", cause="unread: a number of a line a row names is unread")
        elif open_side:
            answer.update(status="undecided", cause=f"side: {side_cause}")
        elif not counted or (low == 0 and high == 0) or (broken == "cancels-is-lacked" and low + high == 0):
            if broken == "nothing-is-zero":
                answer.update(status="value", low=Fraction(0), high=Fraction(0), avg=Fraction(0))
            else:
                answer.update(status="lacked")
            cites.add("nothing")
        else:
            # C94: summed slot-wise; `avg` the mean of the two totals, exact (pseudo.rs).
            answer.update(status="value", low=low, high=high, avg=(low + high) / 2)
            cites.add("range")
            if low + high == 0:
                cites.add("cancels")  # ruling 3: nothing only where low and high are both nothing
            if broken == "low-for-avg":
                answer["avg"] = low
        if answer["status"] in ("undecided", "unruled") and counted:
            answer.update(low=low, high=high, avg=(low + high) / 2)  # what was readable: the subtotal
        answer["cites"] = sorted(cites)
        answers[total] = answer
    return answers


# ---- the site, read beside the reading ------------------------------------------------------------

def local_ids():
    """The ids of the site's `(Local)` twins, or None where the stats file is absent."""
    if not TRADE_STATS.exists():
        return None
    return {e["id"] for g in json.loads(TRADE_STATS.read_text())["result"] for e in g["entries"]
            if e["text"].endswith(" (Local)")}


def site_twin(entry, local):
    if local is None or not entry.get("hash"):
        return "unknown"
    return "local" if entry["hash"].removeprefix("stat.") in local else "global"


def twin_taken(tables, item, array):
    """What the reading takes an unsuffixed line for, by the item's side and the line's array."""
    if array in LEFT_OUT:
        return "left out"
    return {"weapon": "local", "not a weapon": "global", None: "open"}[tables.side(item)[0]]


def shown(text, description):
    """The numbers a pseudo's line shows (tools/trade-evidence.py, `shown`)."""
    number = r"[+-]?\d+(?:\.\d+)?"
    pattern = re.escape(text).replace(r"\+\#", r"\#").replace(r"\#", f"({number})")
    found = re.fullmatch(pattern, description)
    return [Fraction(n) for n in found.groups()] if found else None


def asked(query):
    """(pseudo id, the site says the item has it not) (tools/trade-evidence.py, `asked`)."""
    return [(f["id"], g["type"] == "not") for g in query.get("stats", []) for f in g.get("filters", [])
            if f["id"].startswith("pseudo.")]


def captured_facts(raw):
    """A fetched item as a private item carries it; the ids kept apart, by line index."""
    lines, beside = {}, []
    for array, entries in raw["lines"].items():
        if array == "pseudoMods":
            continue
        lines[array] = [(e["description"], e.get("flags")) for e in entries]
        beside += entries
    return ({"realm": "pc", "base": raw.get("baseType"), "rarity": raw.get("rarity"),
             "name": raw.get("name") if raw.get("rarity") == "Unique" else "", "lines": lines}, beside)


PASSED_OVER = Counter()


def copy_items():
    """Every live item of the copy, by a key that holds no id: (facts, the realm)."""
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
            # an element with no text is no line and is passed over, as scripts/ranged-census.py passes
            # it (the copy's `ultimatumMods` hold `{tier, type}`); the run counts them
            lines[array] = readable([(e.get("description"), e.get("flags")) if isinstance(e, dict)
                                     else (e, None) for e in entries])
            PASSED_OVER[array] += len(entries) - len(lines[array])
        out.append({"realm": realm, "base": body.get("baseType"), "rarity": rarity or "",
                    "name": body.get("name", "") if rarity == "Unique" else "",
                    "type_line": body.get("typeLine"), "lines": lines})
    out.sort(key=sort_key)
    return out


def sort_key(item):
    return (item.get("base") or "", item.get("rarity") or "", item.get("name") or "",
            json.dumps(item["lines"], sort_keys=True, default=str))


def held_key(item):
    return (item.get("base"), item.get("rarity"),
            tuple(sorted(str(t) for entries in item["lines"].values() if isinstance(entries, list)
                         for t, _ in entries)))


def readable(entries):
    """The elements of a lines array that are text; one that is not is `read`'s to answer for."""
    return [(t, f) for t, f in entries if isinstance(t, str)]


def flags_shown(flags):
    return ";".join(sorted(k for k, v in (flags or {}).items() if v is True))


def lines_text(item, only=None):
    """An item's lines as written in a row: `array[flags]: text`, a line break shown as ` ⏎ `."""
    out, others = [], 0
    for array, entries in item["lines"].items():
        if not isinstance(entries, list):
            out.append(f"{array}: (no array: {json.dumps(entries)})")
            continue
        for text, flags in entries:
            if not isinstance(text, str):
                out.append(f"{array}: (no text: {json.dumps(text)})")
                continue
            if only is not None and not any(template(r) in only or "Adds " in r for r in text.split("\n")):
                others += 1
                continue
            unread = {k: v for k, v in (flags or {}).items() if not isinstance(v, bool)}
            mark = flags_shown(flags) + ("".join(f";{k}={v!r} (unread)" for k, v in unread.items()))
            out.append(f"{array}{'[' + mark.strip(';') + ']' if mark else ''}: " + text.replace("\n", " ⏎ "))
    if others:
        out.append(f"(+{others} lines no row names)")
    return " | ".join(out)


# ---- the composed cases, and what each ruled case asks of the copy --------------------------------

def line(text, array="explicitMods", flags=None):
    return array, text, flags


def composed(realm, base, rarity, *lines, name="", arrays=None):
    out = {}
    for array, text, flags in lines:
        out.setdefault(array, []).append((text, flags))
    out.update(arrays or {})
    return {"realm": realm, "base": base, "rarity": rarity, "name": name, "lines": out}


ABSENT_BASE = "A Base The Class Table Lacks"

# The ruled cases that are composed: (case, the sentence it answers, the item, marks).
RULED_COMPOSED = [
    ("r-category-rod",
     "DESIGN.md, C94 — the ranged family: on a weapon the line is the weapon's own — a weapon being what "
     "the category table holds under `weapon`, a Fishing Rod among them",
     composed("pc", "Fishing Rod", "Rare", line("Adds 4 to 9 Cold Damage")), ""),
    ("r-category-frame",
     "class.rs: a base under several classes is read by the frame; category.rs, J17: an id every "
     "candidate gives holds — Energy Blade, no gem by its rarity, is a weapon under each sword class",
     composed("pc", "Energy Blade", "Normal", line("Adds 6 to 114 Lightning Damage")), ""),
    ("r-nothing-zero",
     "totals.rs, C94; table-changes.csv, rule: a total whose lines sum to nothing is absent, never 0",
     composed("pc", "Ruby Ring", "Rare", line("Adds 0 to 0 Fire Damage")), ""),
    ("r-nothing-cancels",
     "ruling 3: a range whose two numbers cancel in the average alone is a value, `low` and `high` not "
     "nothing and `avg` 0",
     composed("pc", "Ruby Ring", "Rare", line("Adds -5 to 5 Fire Damage")), ""),
    ("r-unread-array",
     "totals.rs, C94: a source a row admits, unread, leaves a subtotal marked incomplete, never a total",
     composed("pc", "Ruby Ring", "Rare", line("Adds 3 to 8 Fire Damage", "implicitMods"),
              arrays={"explicitMods": "unreadable"}), ""),
    ("r-unread-flag",
     "J32: a line whose flag is no yes or no counts by its text, no row asking a flag",
     composed("pc", "Driftwood Wand", "Rare", line("Adds 3 to 8 Fire Damage", flags={"crafted": "yes"}),
              line("Adds 2 to 5 Cold Damage")), ""),
    ("r-eldritch-form",
     "LEDGER 9c: a row's eldritch forms are the row (`include the eldritch mods`)",
     composed("pc", "Iron Gauntlets", "Rare",
              line("While a Unique Enemy is in your Presence, Adds 7 to 12 Fire Damage to Attacks",
                   "implicitMods"),
              line("While a Pinnacle Atlas Boss is in your Presence, Adds 2 to 30 Lightning Damage to Spells",
                   "implicitMods")), ""),
    ("r-rows-of-a-mod",
     "LEDGER 9c4: a mod displayed over several rows contributes by a row of it and stays one occurrence",
     composed("pc", "Ruby Ring", "Rare", line("Adds 10 to 20 Fire Damage\n25% chance to Ignite")),
     "line break"),
    ("r-side-open-unsuffixed",
     "ruling 2; decisions/search.md, C93: what cannot be established — a base the class table lacks — makes "
     "only the terms that needed it undecided: the attacks' totals answer, the plain and the spells' do not",
     composed("pc", ABSENT_BASE, "Rare", line("Adds 5 to 10 Fire Damage"),
              line("Adds 1 to 2 Cold Damage to Spells")), ""),
    ("r-side-open-scoped",
     "decisions/search.md, C93: only the terms that needed it — an item whose side is open and whose lines "
     "no twin row names is answered",
     composed("pc", ABSENT_BASE, "Rare", line("Adds 5 to 10 Fire Damage to Attacks")), ""),
    ("r-side-open-no-base",
     "ruling 2; decisions/search.md, C93; class.rs: the item has no base — its attacks' totals answer",
     composed("pc", None, "Rare", line("Adds 5 to 10 Chaos Damage")), ""),
]


def ruled_from_copy(tables, uniques_limit):
    """(case, the sentence, a predicate over a copy item and its answers, marks, the composed stand-in)."""
    unsuffixed = tables.twinned

    def fam(item):
        return [(a, t, f, template(r)) for a, entries in item["lines"].items() if isinstance(entries, list)
                for t, f in readable(entries) for r in t.split("\n") if template(r) in tables.templates]

    def one_unsuffixed(item, side, array="explicitMods", flags=""):
        f = fam(item)
        return (item["realm"] == "pc" and tables.side(item)[0] == side and len(f) == 1
                and f[0][3] in unsuffixed and f[0][0] == array and flags_shown(f[0][2]) == flags)

    def odd(item):
        f = fam(item)
        nums = numbers(f[0][1])
        return len(nums) >= 2 and (nums[0] + nums[1]) % 2 == 1

    return [
        ("r-own", "DESIGN.md, C94 — the ranged family: a weapon's own line feeds the attacks' total alone",
         lambda i: i["rarity"] == "Rare" and one_unsuffixed(i, "weapon"), ""),
        ("r-other", "DESIGN.md, C94 — the ranged family: the other twin feeds the plain, the attacks' and "
                    "the spells'",
         lambda i: one_unsuffixed(i, "not a weapon"), ""),
        ("r-left-out-crucible", "ruling 1: a total leaves out a crucible node's lines — a weapon whose only "
                                "line of the family is a crucible node's lacks every total",
         lambda i: one_unsuffixed(i, "weapon", "crucibleMods"), ""),
        ("r-left-out-flask", "ruling 1: a total leaves out a flask's own lines — a flask of the copy, its "
                             "own lines in `utilityMods`",
         lambda i: i["realm"] == "pc" and isinstance(i["lines"].get("utilityMods"), list)
         and len(i["lines"]["utilityMods"]) > 0, ""),
        ("r-category-implicit", "DESIGN.md, C94 — the ranged family; J7: on a weapon the line is the "
                                "weapon's own — an implicit",
         lambda i: i["realm"] == "pc" and tables.side(i)[0] == "weapon"
         and any(a == "implicitMods" and t in unsuffixed for a, _, _, t in fam(i)), ""),
        ("r-category-enchant", "DESIGN.md, C94 — the ranged family; J7: on a weapon the line is the "
                               "weapon's own — an enchant",
         lambda i: i["realm"] == "pc" and tables.side(i)[0] == "weapon"
         and any(a == "enchantMods" and t in unsuffixed for a, _, _, t in fam(i)), ""),
        ("r-category-crafted", "totals.rs, C94: a row admits any flag — a crafted line on a weapon is the "
                               "weapon's own as any other",
         lambda i: i["realm"] == "pc" and tables.side(i)[0] == "weapon"
         and any(flags_shown(f) == "crafted" and t in unsuffixed for _, _, f, t in fam(i)), ""),
        ("r-category-fractured", "totals.rs, C94: a row admits any flag — a fractured line",
         lambda i: i["realm"] == "pc" and tables.side(i)[0] == "weapon"
         and any(flags_shown(f) == "fractured" and t in unsuffixed for _, _, f, t in fam(i)), ""),
        ("r-avg", "DESIGN.md, C94 — the ranged family: a ranged total asked with no slot word is its `avg`, "
                  "the one number the site shows — here a half",
         lambda i: one_unsuffixed(i, "not a weapon") and odd(i), ""),
        ("r-limit", "DESIGN.md, C94 — the ranged family: every ranged total prints the limit — a global line "
                    "on a weapon is read as the weapon's own; here a unique weapon whose line the export "
                    "cannot place and no capture holds",
         lambda i: i["realm"] == "pc" and i["rarity"] == "Unique"
         and any((i["name"], i["base"], r) in uniques_limit for entries in i["lines"].values()
                 if isinstance(entries, list) for t, _ in readable(entries) for r in t.split("\n")), "limit"),
        ("r-nothing", "totals.rs, C94; table-changes.csv, rule: a total of nothing is lacked — no line a "
                      "row names on the item",
         lambda i: i["realm"] == "pc" and i["rarity"] == "Rare" and not fam(i)
         and sum(len(e) for e in i["lines"].values() if isinstance(e, list)) >= 3
         and not any("Adds " in t for e in i["lines"].values() if isinstance(e, list) for t, _ in readable(e)), ""),
        ("r-not-a-row", "totals.rs, C94: a readable line absent from every row is not an unknown "
                        "contributor — the total answers its declared definition",
         lambda i: i["realm"] == "pc" and not fam(i)
         and any(template(t).startswith("Adds # to # ") for e in i["lines"].values() if isinstance(e, list)
                 for t, _ in readable(e)), ""),
        ("r-slot-wise", "decisions/search.md, C94: a total over ranged lines is a range, summed slot-wise; "
                        "table-changes.csv, rule: a ranged total is the average of a line's two numbers",
         lambda i: i["realm"] == "pc" and i["rarity"] == "Rare" and tables.side(i)[0] == "weapon"
         and len(fam(i)) == 2 and all(t in unsuffixed and "Physical" not in t and "Chaos" not in t
                                      for _, _, _, t in fam(i))
         and len({t for _, _, _, t in fam(i)}) == 2, ""),
        ("r-reduced", "table-changes.csv, rule: a row's `reduced` spelling is the row, counted below "
                      "nothing — which meets no row of the 21: none says `increased`",
         lambda i: i["realm"] == "pc" and i["rarity"] == "Rare" and not fam(i)
         and any(" reduced " in t for e in i["lines"].values() if isinstance(e, list) for t, _ in readable(e)), ""),
        ("r-realm", "totals.rs, C94: no definition for the item's realm is never a zero",
         lambda i: i["realm"] not in tables.realms
         and any(template(MARKUP.sub(lambda m: m.group(2) or m.group(1), r)) in unsuffixed
                 for e in i["lines"].values() if isinstance(e, list) for t, _ in readable(e) for r in t.split("\n")),
         ""),
    ]


# The worked cases: each cell's arithmetic written by hand before the function is run on it
# (the plan, rule 3). {total: (low, high, avg, the arithmetic)}; a total not listed is lacked.
def worked_cases():
    w1 = composed("pc", "Driftwood Wand", "Rare", line("Adds 10 to 21 Fire Damage"))
    one = ("10", "21", "15.5", "one line: low 10; high 21; avg (10 + 21) / 2 = 15.5")
    w2 = composed("pc", "Ruby Ring", "Rare", line("Adds 10 to 21 Fire Damage"))
    return [
        ("w-fire-on-a-weapon",
         "the plan's closing row, 9d-C: `Adds # to # Fire Damage` on a weapon counted toward attacks alone",
         w1, {t: one for t in ("adds_fire_to_attacks", "adds_elemental_to_attacks", "adds_damage_to_attacks")}),
        ("w-fire-on-no-weapon",
         "the same line on an item that is no weapon: the other twin, the plain, the attacks' and the spells'",
         w2, {t: one for t in ("adds_fire", "adds_elemental", "adds_damage",
                               "adds_fire_to_attacks", "adds_elemental_to_attacks", "adds_damage_to_attacks",
                               "adds_fire_to_spells", "adds_elemental_to_spells", "adds_damage_to_spells")}),
    ] + WORKED_MORE


# What the captured, the ruled and the two worked cases above leave uncounted is six rows over three
# templates (the script prints them where this list is emptied): one composed item carries the three.
WORKED_MORE = [
    ("w-rows-no-case-counted",
     "the rows no other case counts: `Adds # to # Chaos Damage to Attacks`, `Adds # to # Physical Damage to "
     "Spells` and `Adds # to # Chaos Damage to Spells`, each under its own total and the all-damage one",
     composed("pc", "Ruby Ring", "Rare", line("Adds 3 to 8 Chaos Damage to Attacks"),
              line("Adds 4 to 11 Physical Damage to Spells"), line("Adds 6 to 9 Chaos Damage to Spells")),
     {"adds_chaos_to_attacks": ("3", "8", "5.5", "one line: low 3; high 8; avg (3 + 8) / 2 = 5.5"),
      "adds_damage_to_attacks": ("3", "8", "5.5", "the chaos line to attacks alone: low 3; high 8; avg "
                                                   "(3 + 8) / 2 = 5.5"),
      "adds_physical_to_spells": ("4", "11", "7.5", "one line: low 4; high 11; avg (4 + 11) / 2 = 7.5"),
      "adds_chaos_to_spells": ("6", "9", "7.5", "one line: low 6; high 9; avg (6 + 9) / 2 = 7.5"),
      "adds_damage_to_spells": ("10", "20", "15", "two lines to spells: low 4 + 6 = 10; high 11 + 9 = 20; "
                                                   "avg (10 + 20) / 2 = 15")}),
]


# ---- the run --------------------------------------------------------------------------------------

HEADER = ["case", "kind", "source", "same_as", "marks", "realm", "base", "rarity", "unique", "lines",
          "total", "status", "low", "high", "avg", "cause_or_readings", "counted", "site", "agrees",
          "twin_site_and_reading", "rests_on", "by_hand", "hand_agrees"]


def answer_cells(answer):
    return [answer["status"], plain(answer["low"]), plain(answer["high"]), plain(answer["avg"])]


def cause_text(answer):
    if answer["status"] == "unruled":
        key = answer["cause"].split(" ")[0]
        return f"{answer['cause']}: {UNRULED[key]}"
    return answer["cause"]


def counted_text(answer):
    return " | ".join(f"{a}{'[' + f + ']' if f else ''}: {row} <- {name}"
                      for (_, a, f, row, name, _) in answer["counted"])


def compare(answer, site):
    """Whether the reading and what the site showed agree: site is a list of shown values, [] for none."""
    if answer["status"] in ("undecided", "unruled"):
        return "not comparable"
    if answer["status"] == "lacked":
        return "yes" if not site else "no"
    return "yes" if site == [[answer["avg"], answer["avg"]]] else "no"


def captured_cells(tables, captures, broken=None):
    """Every captured item under the reading: yields (case, search, index, raw, facts, beside, answers,
    {total: what the site showed})."""
    for name, capture in captures.items():
        named = [(tables.by_site[p], no) for p, no in asked(capture["returned_query"]) if p in tables.by_site]
        if not named:
            continue
        for index, raw in enumerate(capture["fetched"] or []):
            facts, beside = captured_facts(raw)
            answers = read(tables, facts, broken)
            site = {}
            for total, site_says_not in named:
                values = [v for e in raw["lines"].get("pseudoMods", [])
                          if (v := shown(tables.totals[total][1], e["description"])) is not None]
                if site_says_not and values:
                    raise SystemExit(f"{name}[{index}] shows a pseudo its search excluded")
                site[total] = values
            yield f"{name}[{index}]", name, index, raw, facts, beside, answers, site


def disagreements(tables, captures, broken=None):
    cells = disagree = open_ = 0
    for _, _, _, _, _, _, answers, site in captured_cells(tables, captures, broken):
        for total, values in site.items():
            cells += 1
            verdict = compare(answers[total], values)
            disagree += verdict == "no"
            open_ += verdict == "not comparable"
    return cells, disagree, open_


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

    absent = [str(p.relative_to(ROOT)) for p in (CAPTURES, STORE, TRADE_STATS, UNIQUES, CENSUS, TRADE_CLASSES)
              if not p.exists()]
    print("local inputs absent: " + ("; ".join(absent) if absent else "none"))
    tables = Tables()
    captures = json.loads(CAPTURES.read_text())["searches"] if CAPTURES.exists() else {}

    if only_break:
        cells, disagree, open_ = disagreements(tables, captures, only_break)
        print(f"break {only_break} ({BREAKS[only_break]}): {disagree} of {cells} captured cells disagree, "
              f"{open_} not comparable; nothing written")
        return 0

    # -- step 2: the rows, and what a weapon is ----------------------------------------------------
    print("\n== the rows (data/table-changes.csv, the 21 totals named adds_…) ==")
    fc, ft = tables.family_changes, tables.family_twins
    print(f"{len(tables.totals)} totals; {fc['add row']} `add row` and {fc['twin']} `twin`, "
          f"{ft['global']} `global` and {ft['local']} `local`")
    print(f"{fc['add row'] + fc['twin']} rows; before the attacks' pairs became rows of `any` twin (ruling 2; "
          f"the brief, 2026-10-08): 96 `add row` and 52 `twin`, 39 `global` and 13 `local`, 148 rows")
    pairs_left = sum(1 for rows in tables.rows.values() for (t, which, w) in rows
                     if which == "local" and (t, "global", w) in rows)
    print(f"totals still holding a `local` and a `global` row of one line at one weight: {pairs_left}")
    below = sum(1 for rows in tables.rows.values() for (_, _, w) in rows if w != 1)
    print(f"{len(tables.templates)} templates the rows name, {len(tables.twinned)} of them under a twin; "
          f"rows at a weight other than 1: {below}; rows whose template says `increased` or `reduced`: "
          f"{sum(1 for t in tables.templates if 'increased' in t or 'reduced' in t)}")
    print(f"`rule` rows of the table of changes: {len(tables.rules)}")
    for rule in tables.rules:
        print(f"  rule: {rule}")

    print("\n== what a weapon is: the category table beside ranged-census.py's file, class by class ==")
    if TRADE_CLASSES.exists():
        with TRADE_CLASSES.open(newline="\n") as fh:
            next(fh)
            trade = {r["class_name"]: r["trade_id"] for r in csv.DictReader(fh) if r["class_name"]}
    else:
        trade = None
        print(f"{TRADE_CLASSES.relative_to(ROOT)} is absent: the comparison is not made")
    same = differ = missing = 0
    for cls in tables.class_names:
        ours = tables.class_is_weapon(cls)
        if trade is None:
            continue
        if cls not in trade:
            missing += 1
            print(f"  {cls}: category table {'weapon' if ours else 'not a weapon'}; the census's file has no "
                  f"row — not compared")
            continue
        theirs = trade[cls].startswith("weapon.")
        verdict = "the same" if ours == theirs else "DIFFER"
        same += ours == theirs
        differ += ours != theirs
        print(f"  {cls}: category table {'weapon' if ours else 'not a weapon'}"
              f" ({tables.leaf.get(cls) or 'no id'}); census's file {'weapon' if theirs else 'not a weapon'}"
              f" ({trade[cls] or 'no id'}) — {verdict}")
    if trade is not None:
        print(f"{len(tables.class_names)} classes of the class table: {same} the same, {differ} differ, "
              f"{missing} with no row in the census's file; "
              f"{sum(1 for c in tables.class_names if tables.class_is_weapon(c))} are weapons by the "
              f"category table")
    both = sorted(b for b, names in tables.classes.items()
                  if len({tables.class_is_weapon(n, b) for n in names}) > 1)
    print(f"bases under classes of both kinds: {len(both)}: {', '.join(both)}")

    local = local_ids()
    rows_out = []
    counts_by_row = defaultdict(Counter)  # row name -> kind -> cases
    kinds = Counter()
    unruled_cells, findings, open_cells = [], [], []

    def emit(case, kind, source, same_as, marks, item, answers, site=None, beside=None, hand=None,
             sentence="", listed=None, only_lines=None):
        twins = ""
        if beside is not None:
            pairs, i = [], 0
            for array, entries in item["lines"].items():
                for text, _ in entries:
                    for row in text.split("\n"):
                        if template(row) in tables.twinned:
                            took = twin_taken(tables, item, array)
                            pairs.append(f"{template(row)}: site {site_twin(beside[i], local)}, reading {took}")
                    i += 1
            twins = " | ".join(pairs)
        for total in tables.totals:
            answer = answers[total]
            if listed is not None and total not in listed and answer["status"] == "lacked":
                continue
            if site is not None and total in site:
                site_text = "none" if not site[total] else " to ".join(plain(v) for v in site[total][0])
                verdict = compare(answer, site[total])
            else:
                site_text, verdict = "not asked", "not asked"
            by_hand = hand_ok = ""
            if hand is not None:
                if total in hand:
                    lo, hi, avg, sums = hand[total]
                    by_hand = f"{sums} -> low {lo}, high {hi}, avg {avg}"
                    hand_ok = "yes" if (answer["status"] == "value" and
                                        (plain(answer["low"]), plain(answer["high"]),
                                         plain(answer["avg"])) == (lo, hi, avg)) else "no"
                else:
                    by_hand = "no row of this total names a line of the item as it is: lacked"
                    hand_ok = "yes" if answer["status"] == "lacked" else "no"
            all_marks = ";".join(m for m in ([marks] if marks else []) + answer["marks"] if m)
            # a captured row names the steps by key, the sentences being CITES above; the others in full
            rests = ("steps: " + ", ".join(answer["cites"]) + " (scripts/ranged-expected.py, CITES)"
                     if kind == "captured" else
                     "; ".join(([sentence] if sentence else []) + [CITES[c] for c in answer["cites"]]))
            rows_out.append([case, kind, source, same_as, all_marks, item["realm"], item.get("base") or "",
                             item.get("rarity") or "", item.get("name") or "",
                             lines_text(item, only_lines), total] + answer_cells(answer)
                            + [cause_text(answer), counted_text(answer), site_text, verdict, twins, rests,
                               by_hand, hand_ok])
            for (_, _, _, _, name, _) in answer["counted"]:
                if answer["status"] == "value":
                    counts_by_row[name][kind] += 1
            if answer["status"] == "unruled":
                unruled_cells.append((case, kind, total, answer["cause"].split(" ")[0]))
            if verdict == "no":
                findings.append((case, total, site_text, answer, item, twins))
            if verdict == "not comparable":
                open_cells.append((case, total, site_text, answer))
        kinds[kind] += 1

    # -- step 3: the captured cases ------------------------------------------------------------------
    print("\n== the captures whose query names a pseudo of the family ==")
    naming = {n: c for n, c in captures.items()
              if any(p in tables.by_site for p, _ in asked(c["returned_query"]))}
    fetched_rows = sum(len(c["fetched"] or []) for c in naming.values())
    empty = [n for n, c in naming.items() if not c["fetched"]]
    round_n = {n: c for n, c in naming.items() if re.fullmatch(r"n\d+", n)}
    print(f"{len(naming)} searches and {fetched_rows} fetched rows; of them round n's: {len(round_n)} searches "
          f"({', '.join(round_n)}) and {sum(len(c['fetched'] or []) for c in round_n.values())} fetched rows")
    found_none = [n for n in empty if not naming[n]["search"].get("total")]
    print(f"read: {len(naming) - len(empty)} searches, every fetched row of each; left out: {len(empty)}, "
          f"each a search that fetched no item, so there is no item to read — {len(found_none)} found "
          f"nothing (the answer is the count) and {len(empty) - len(found_none)} found items and fetched "
          f"none: " + ", ".join(empty))
    other_named = Counter(p for c in naming.values() for p, _ in asked(c["returned_query"])
                          if p not in tables.by_site)
    print(f"pseudos those queries name outside the family, not read here: {len(other_named)}")
    first, captured_keys, captured_uniques = {}, set(), set()
    cells = Counter()
    asked_by_total = Counter()
    groups_asked = defaultdict(lambda: {"cells": 0, "agree": 0, "items": set()})
    for case, name, index, raw, facts, beside, answers, site in captured_cells(tables, captures):
        key = json.dumps({k: (dict(v, pseudoMods=None) if k == "lines" else v)
                          for k, v in raw.items() if k != "extended"}, sort_keys=True)
        same_as = first.get(key, "")
        first.setdefault(key, case)
        captured_keys.add(held_key(facts))
        if facts["name"]:
            captured_uniques.add((facts["name"], facts["base"]))
        emit(case, "captured", f"data/captures.json, {name}", same_as, "", facts, answers, site=site,
             beside=beside, listed=set(site), only_lines=tables.templates)
        side = tables.side(facts)[0] or "open"
        for total, values in site.items():
            verdict = compare(answers[total], values)
            cells[verdict] += 1
            asked_by_total[total] += 1
            named_here = {t for (t, _, _) in tables.rows[total]}
            seen = set()
            for array, entries in facts["lines"].items():
                for text, flags in entries:
                    for row in text.split("\n"):
                        g = (template(row), side, array, flags_shown(flags))
                        if g[0] in named_here and g not in seen:
                            seen.add(g)
                            groups_asked[g]["cells"] += 1
                            groups_asked[g]["agree"] += verdict == "yes"
                            groups_asked[g]["items"].add(first[key])
    kinds_captured_distinct = len(first)
    print(f"captured cases: {kinds['captured']} fetched rows, {kinds_captured_distinct} distinct items, "
          f"{kinds['captured'] - kinds_captured_distinct} repeats named beside their first row")
    total_cells = sum(cells.values())
    print(f"captured cells — an item under a total its search named: {total_cells}; the reading and the site "
          f"agree on {cells['yes']}, disagree on {cells['no']}, not comparable (the reading undecided or "
          f"unruled) on {cells['not comparable']}")
    shows = sum(1 for r in rows_out if r[1] == "captured" and r[17] not in ("not asked", "none"))
    print(f"of those cells the site showed a value on {shows} and none on {total_cells - shows}")
    print("captured cells by total: " + "; ".join(f"{t} {asked_by_total[t]}" for t in tables.totals))
    side_captured = Counter()
    twin_pairs = Counter()
    for key, case in first.items():
        raw = json.loads(key)
        facts, beside = captured_facts(raw)
        side_captured[tables.side(facts)[0] or "open"] += 1
        i = 0
        for array, entries in facts["lines"].items():
            for text, _ in entries:
                for row in text.split("\n"):
                    if template(row) in tables.twinned:
                        took = twin_taken(tables, facts, array)
                        twin_pairs[(site_twin(beside[i], local), took)] += 1
                i += 1
    print("distinct captured items by side: " + "; ".join(f"{k} {v}" for k, v in sorted(side_captured.items())))
    print("unsuffixed lines on distinct captured items, the site's twin by the reading's: "
          + "; ".join(f"site {s} / reading {r}: {n}" for (s, r), n in sorted(twin_pairs.items())))
    breaks_in_captures = sum(1 for key in first for a, es in json.loads(key)["lines"].items() if a != "pseudoMods"
                             for e in es if "\n" in e["description"]
                             and any(template(r) in tables.templates for r in e["description"].split("\n")))
    print(f"captured lines of the family whose text holds a line break: {breaks_in_captures}")

    # -- the copy ------------------------------------------------------------------------------------
    copy = copy_items() if STORE.exists() else []
    if not STORE.exists():
        print(f"\n{STORE.relative_to(ROOT)} is absent: no ruled case is an item of the copy, and steps 5 and 6 "
              f"are not made")

    # -- the ruled cases -----------------------------------------------------------------------------
    print("\n== the ruled cases ==")
    limit = set()
    if UNIQUES.exists():
        with UNIQUES.open(newline="\n") as fh:
            next(fh)
            for r in csv.DictReader(fh):
                if (r["weapon"] == "weapon" and r["export_says"].startswith("cannot say")
                        and r["site_says"] == "not captured" and r["realm"] == "pc"):
                    limit.add((r["unique"], r["base"], r["line"]))
    used = set()
    for case, sentence, wanted, marks in ruled_from_copy(tables, limit):
        # an item the captures do not hold: not by its lines, and a unique not by its name either
        found = next((i for i in copy if wanted(i) and held_key(i) not in captured_keys
                      and (i["name"], i["base"]) not in captured_uniques
                      and sort_key(i) not in used), None)
        if found is not None:
            used.add(sort_key(found))
        if found is None:
            print(f"  {case}: the copy holds no such item the captures do not — NO CASE; {sentence}")
            continue
        emit(case, "ruled", "copy", "", marks, found, read(tables, found), sentence=sentence)
        print(f"  {case}: an item of the copy ({found['rarity'] or 'no rarity'} {found['base']}"
              f"{', ' + found['name'] if found['name'] else ''}) — {sentence}")
    for case, sentence, item, marks in RULED_COMPOSED:
        if item["base"] == ABSENT_BASE and ABSENT_BASE in tables.classes:
            raise SystemExit("the composed absent base is in the class table")
        emit(case, "ruled", "composed", "", marks, item, read(tables, item), sentence=sentence)
        print(f"  {case}: composed — {sentence}")

    # -- the worked cases ----------------------------------------------------------------------------
    print("\n== the worked cases: the hand's arithmetic beside the function's ==")
    for case, sentence, item, hand in worked_cases():
        answers = read(tables, item)
        before = len(rows_out)
        emit(case, "worked", "composed", "", "", item, answers, hand=hand, sentence=sentence)
        mine = rows_out[before:]
        wrong = [r[10] for r in mine if r[22] != "yes"]
        print(f"  {case}: {len(mine)} cells, {sum(1 for r in mine if r[11] == 'value')} a value by hand and "
              f"{sum(1 for r in mine if r[11] == 'lacked')} lacked; the function gave what the hand did on "
              f"{len(mine) - len(wrong)}" + (f"; NOT on {', '.join(wrong)}" if wrong else ""))

    print("\n== the cases by kind ==")
    by_kind_rows = Counter(r[1] for r in rows_out)
    by_source = Counter((r[1], r[2].split(",")[0]) for r in {(r[0], r[1], r[2]) for r in rows_out})
    print("; ".join(f"{k}: {kinds[k]} cases, {by_kind_rows[k]} rows" for k in ("captured", "ruled", "worked")))
    print(f"ruled cases: {by_source[('ruled', 'copy')]} items of the copy, "
          f"{by_source[('ruled', 'composed')]} composed; worked: {by_source[('worked', 'composed')]} composed")
    status = Counter((r[1], r[11]) for r in rows_out)
    print("rows by kind and status: " + "; ".join(f"{k} {s} {n}" for (k, s), n in sorted(status.items())))

    print("\n== each row of the 21 totals: the cases that count it, by kind ==")
    uncounted = []
    for total in tables.totals:
        for (t, which, _) in tables.rows[total]:
            name = f"{total} · {t} · {which}"
            got = Counter()
            for counted_name, c in counts_by_row.items():
                # a row counted with the side open is named `local or global`
                if counted_name == name:
                    got.update(c)
            if not got:
                uncounted.append(name)
            print(f"  {name}: captured {got['captured']}, ruled {got['ruled']}, worked {got['worked']}")
    n_rows = sum(len(r) for r in tables.rows.values())
    print(f"{n_rows} rows; counted by a case: {n_rows - len(uncounted)}; counted by none: {len(uncounted)}")
    for name in uncounted:
        print(f"  NO CASE counts: {name}")

    # -- step 4: disagreements, and what the agreement is worth --------------------------------------
    print("\n== disagreements: a captured cell where the reading and the site differ ==")
    print(f"{len(findings)} cells")
    for case, total, site_text, answer, item, twins in findings:
        print(f"  {case} {total}: the site {site_text}; the reading {answer['status']} "
              f"{plain(answer['avg'])} — {item['rarity']} {item['base']}"
              f"{', ' + item['name'] if item.get('name') else ''}; {twins}")
    print(f"not comparable: {len(open_cells)} cells")
    for case, total, site_text, answer in open_cells:
        print(f"  {case} {total}: the site {site_text}; the reading {answer['status']}: {answer['cause']}")

    print("\n== the reading broken once for each rule: captured cells that then disagree ==")
    alone = []
    for name, what in BREAKS.items():
        n, disagree, open_ = disagreements(tables, captures, name)
        print(f"  --break {name} ({what}): {disagree} of {n} disagree")
        if disagree == len(findings):
            alone.append(name)
    print(f"rules whose break no capture notices — resting on the ruling alone: {len(alone)}: "
          + ", ".join(alone))

    print("\n== the unruled cells ==")
    by_question = defaultdict(list)
    for case, kind, total, key in unruled_cells:
        by_question[key].append((case, kind, total))
    print(f"{len(unruled_cells)} cells under {len(by_question)} questions")
    for key, cells_ in by_question.items():
        cases = sorted({c for c, _, _ in cells_})
        kinds_ = Counter(k for _, k, _ in cells_)
        print(f"  {key}: {len(cells_)} cells on {len(cases)} cases ("
              + ", ".join(f"{k} {n}" for k, n in sorted(kinds_.items())) + f"): {', '.join(cases[:6])}"
              + (" …" if len(cases) > 6 else ""))
        print(f"    totals: {', '.join(sorted({t for _, _, t in cells_}))}")
        for reading in UNRULED[key].split(" || "):
            print(f"    {reading}")

    # -- step 5: what no capture has asked -----------------------------------------------------------
    not_asked = []
    group_rows = []
    if copy:
        print("\n== what no capture has asked: the copy's lines by template, side, array and flags ==")
        groups = defaultdict(lambda: {"lines": 0, "items": 0})
        lacking_base = 0
        for item in copy:
            if item["realm"] != "pc":
                continue
            side = tables.side(item)[0] or "open"
            seen = set()
            for array, entries in item["lines"].items():
                if not isinstance(entries, list):
                    continue
                for text, flags in readable(entries):
                    for row in text.split("\n"):
                        t = template(row)
                        if t in tables.templates:
                            g = (t, side, array, flags_shown(flags))
                            groups[g]["lines"] += 1
                            if g not in seen:
                                seen.add(g)
                                groups[g]["items"] += 1
            if seen and side == "open":
                lacking_base += 1
        print(f"live pc items carrying a line a row names whose side is open (a base the class table lacks, "
              f"or none): {lacking_base}")

        def question(g):
            t, side, array, flags = g
            where = f"{array}{' (' + flags + ')' if flags else ''}"
            what = {"weapon": "a weapon", "not a weapon": "an item that is no weapon",
                    "open": "an item the class table cannot class"}[side]
            if array in LEFT_OUT:
                named = sorted(n for n, rows in tables.rows.items() if any(r[0] == t for r in rows))
                return ("ruling 1: a total leaves out the array's lines, whatever the line",
                        f"does any pseudo of the family ({', '.join(tables.totals[n][0] for n in named)}) "
                        f"count `{t}` in {where} on {what}? The reading: none")
            if t in tables.twinned:
                total = next(n for n, rows in tables.rows.items() if len(rows) == 1 and rows[0][0] == t)
                expects = {"weapon": "none, the attacks' pseudo showing the line's average",
                           "not a weapon": "the line's average",
                           "open": "undecided, the attacks' pseudo showing the line's average"}[side]
                rule = ("J7: on a weapon the line is the weapon's own, whatever its array or flag"
                        if side == "weapon" else
                        "the paragraph: the other twin feeds the plain, the attacks' and the spells'; "
                        "totals.rs: any source, any flag")
            else:
                own = [n for n, (_, text) in tables.totals.items() if text in t and "elemental" not in n
                       and n != "adds_damage" and not n.startswith("adds_damage_")]
                own = [n for n in own if any(r[0] == t for r in tables.rows[n])]
                total = sorted(own, key=lambda n: -len(tables.totals[n][1]))[0] if own else sorted(
                    n for n, rows in tables.rows.items() if any(r[0] == t for r in rows))[0]
                expects = "the line's average"
                rule = "totals.rs: a row admits any source and any flag; the row is the table's"
            return rule, (f"does `{tables.totals[total][1]}` ({tables.totals[total][0]}) show on {what} "
                          f"carrying `{t}` in {where}? The reading: {expects}")

        asked_n = 0
        for g in sorted(groups):
            held = groups[g]
            got = groups_asked.get(g)
            rule, q = question(g)
            group_rows.append(["lines", g[0], g[1], g[2], g[3], held["items"], held["lines"], "", "", "",
                               got["cells"] if got else 0, len(got["items"]) if got else 0,
                               got["agree"] if got else 0, "asked" if got else "not asked", rule,
                               "" if got else q])
            if got:
                asked_n += 1
                print(f"  asked: {g[0]} · {g[1]} · {g[2]}{' (' + g[3] + ')' if g[3] else ''}: copy "
                      f"{held['lines']} lines on {held['items']} items; captured cells {got['cells']} on "
                      f"{len(got['items'])} distinct items, {got['agree']} agreeing")
            else:
                not_asked.append((g, held, rule, q))
        print(f"{len(groups)} groups on the copy: {asked_n} asked by a capture, {len(not_asked)} not asked")
        print(f"groups a capture asked that the copy does not hold: "
              f"{sum(1 for g in groups_asked if g not in groups)}")
        for g, held, rule, q in not_asked:
            print(f"  NOT ASKED: {g[0]} · {g[1]} · {g[2]}{' (' + g[3] + ')' if g[3] else ''}: "
                  f"{held['lines']} lines on {held['items']} items")
            print(f"    rests on: {rule}")
            print(f"    a sitting would ask: {q}")
        lines_na = sum(h["lines"] for _, h, _, _ in not_asked)
        lines_all = sum(h["lines"] for h in groups.values())
        print(f"lines of the copy in a group not asked: {lines_na} of {lines_all}")

        def kind_of(t):
            scope = re.sub(r"^.*Adds # to # \w+ Damage", "", t).strip() or "unsuffixed"
            return ("eldritch form, " if t.startswith("While a ") else "") + scope

        def type_of(t):
            return re.search(r"Adds # to # (\w+) Damage", t).group(1)

        print("the groups not asked, the damage type folded — by kind of line, side, array and flags:")
        folded = defaultdict(lambda: {"lines": 0, "items": 0, "types": [], "asked": []})
        for g in sorted(groups):
            key = (kind_of(g[0]), g[1], g[2], g[3])
            if g in groups_asked:
                folded[key]["asked"].append(type_of(g[0]))
            else:
                folded[key]["lines"] += groups[g]["lines"]
                folded[key]["items"] += groups[g]["items"]
                folded[key]["types"].append(type_of(g[0]))
        by_rule = defaultdict(lambda: {"groups": 0, "lines": 0, "where": Counter()})
        asks = {
            "ruling 1, a line in an array every total leaves out, of a type no capture asked":
                "does any pseudo of the family count the line there? The reading: none",
            "J7, a weapon's own line in an array or under a flag no capture asked":
                "does the plain pseudo of the line's type show on a weapon carrying the unsuffixed line "
                "there? The reading: none, the attacks' pseudo showing the line's average",
            "the other twin in an array no capture asked":
                "does the plain pseudo of the line's type show on an item that is no weapon carrying the "
                "unsuffixed line there? The reading: the line's average",
            "a row of `any` twin (totals.rs: any source, any flag) where no capture asked":
                "does the pseudo of the line's own scope and type show the line's average there? The "
                "reading: yes",
        }
        for key in sorted(folded, key=lambda k: (k[0] != "unsuffixed", k)):
            f = folded[key]
            if not f["types"]:
                continue
            rule = ("ruling 1, a line in an array every total leaves out, of a type no capture asked"
                    if key[2] in LEFT_OUT else
                    "J7, a weapon's own line in an array or under a flag no capture asked"
                    if key[0] == "unsuffixed" and key[1] == "weapon" else
                    "the other twin in an array no capture asked" if key[0] == "unsuffixed" else
                    "a row of `any` twin (totals.rs: any source, any flag) where no capture asked")
            by_rule[rule]["groups"] += len(f["types"])
            by_rule[rule]["lines"] += f["lines"]
            where = key[2] + (f" ({key[3]})" if key[3] else "")
            by_rule[rule]["where"][where if key[0] == "unsuffixed" and key[2] not in LEFT_OUT
                                   else f"{key[0]}, {key[1]}, {where}"] += f["lines"]
            print(f"  {key[0]} · {key[1]} · {key[2]}{' (' + key[3] + ')' if key[3] else ''}: {f['lines']} lines "
                  f"on {f['items']} items; not asked of {', '.join(f['types'])}"
                  + (f"; asked of {', '.join(f['asked'])}" if f["asked"] else "; asked of no type"))
        for rule, n in by_rule.items():
            print(f"  by rule — {rule}: {n['groups']} groups, {n['lines']} lines — "
                  + "; ".join(f"{w} {c}" for w, c in sorted(n["where"].items(), key=lambda kv: (-kv[1], kv[0])))
                  + f". A sitting would ask: {asks[rule]}")
        negative = sum(1 for i in copy for e in i["lines"].values() if isinstance(e, list) for t, _ in readable(e)
                       for r in t.split("\n") if template(r) in tables.templates
                       and any(n < 0 for n in numbers(r)))
        negative_c = sum(1 for key in first for a, es in json.loads(key)["lines"].items() if a != "pseudoMods"
                         for e in es for r in e["description"].split("\n")
                         if template(r) in tables.templates and any(n < 0 for n in numbers(r)))
        print(f"lines a row names holding a number below nothing: {negative} on the copy, {negative_c} on the "
              f"distinct captured items")

    # -- step 6: the function over the copy ----------------------------------------------------------
    copy_rows = []
    if copy:
        print("\n== the copy: the function over every live pc item ==")
        tally = defaultdict(lambda: {"items": 0, "low": Fraction(0), "high": Fraction(0), "avg": Fraction(0)})
        pc = [i for i in copy if i["realm"] == "pc"]
        any_value = 0
        for item in pc:
            answers = read(tables, item)
            any_value += any(a["status"] != "lacked" for a in answers.values())
            for total, a in answers.items():
                if a["status"] == "lacked":
                    continue
                key = (total, a["status"], a["cause"] if a["status"] != "value" else "")
                tally[key]["items"] += 1
                if a["status"] == "value":
                    for slot in ("low", "high", "avg"):
                        tally[key][slot] += a[slot]
        for total in tables.totals:
            for key in sorted(k for k in tally if k[0] == total):
                t = tally[key]
                value = key[1] == "value"
                copy_rows.append(["total", total, key[1], key[2], "", t["items"], "",
                                  plain(t["low"]) if value else "", plain(t["high"]) if value else "",
                                  plain(t["avg"]) if value else "", "", "", "", "", "", ""])
            if not any(k[0] == total for k in tally):
                copy_rows.append(["total", total, "value", "", "", 0, "", "0", "0", "0", "", "", "", "", "", ""])
        print(f"{len(pc)} live pc items; {any_value} have a total of the 21 or leave one open")
        no_array = sum(1 for i in pc if any(not isinstance(e, list) for e in i["lines"].values()))
        print(f"live pc items with a `…Mods` array that is no array: {no_array}; elements of a `…Mods` array "
              f"that hold no text, passed over as no line, every live item: "
              + ("; ".join(f"{a} {n}" for a, n in sorted(PASSED_OVER.items()) if n) or "none"))
        for array in LEFT_OUT:
            holding = [i for i in pc if isinstance(i["lines"].get(array), list) and i["lines"][array]]
            named = sum(1 for i in holding for t, _ in readable(i["lines"][array]) for r in t.split("\n")
                        if template(r) in tables.templates)
            print(f"ruling 1 on the copy — live pc items holding `{array}`: {len(holding)}; lines there a row "
                  f"of the 21 names, left out: {named}")
        for r in copy_rows:
            print(f"  {r[1]}: {r[2]}{' (' + r[3] + ')' if r[3] else ''}: {r[5]} items"
                  + (f"; low {r[7]}, high {r[8]}, avg {r[9]}" if r[2] == "value" else ""))
        print("items undecided or unruled, by cause: "
              + ("; ".join(f"{k[1]} {k[2]}: {v['items']} item-totals" for k, v in sorted(tally.items())
                           if k[1] != "value") or "none"))

        # the census's items: the unsuffixed five, every live item, poe2's markup collapsed
        mine_lines = Counter()
        mine_items = 0
        for item in copy:
            side = tables.side(item)[0] if item["realm"] in tables.realms else None
            word = side or "no class"
            n = 0
            for entries in item["lines"].values():
                if not isinstance(entries, list):
                    continue
                for text, _ in readable(entries):
                    for row in text.split("\n"):
                        if template(MARKUP.sub(lambda m: m.group(2) or m.group(1), row)) in tables.twinned:
                            mine_lines[word] += 1
                            n += 1
            mine_items += n > 0
        theirs = Counter()
        if CENSUS.exists():
            with CENSUS.open(newline="\n") as fh:
                next(fh)
                for r in csv.DictReader(fh):
                    theirs[r["weapon"]] += int(r["lines"])
        print(f"items carrying an unsuffixed line of the family, every live item: {mine_items}, "
              f"{sum(mine_lines.values())} lines — " + "; ".join(f"{k} {v}" for k, v in sorted(mine_lines.items())))
        print("data/ranged-census.csv: " + f"{sum(theirs.values())} lines — "
              + "; ".join(f"{k} {v}" for k, v in sorted(theirs.items()))
              + (" — the same lines by side" if theirs == mine_lines else " — NOT the same"))
        print("the README states 703 lines on 559 items, 573 on a weapon, 129 not, 1 with no class — "
              + ("agree" if (sum(mine_lines.values()), mine_items, mine_lines["weapon"],
                             mine_lines["not a weapon"], mine_lines["no class"]) == (703, 559, 573, 129, 1)
                 else "DO NOT agree"))

    # -- the files -----------------------------------------------------------------------------------
    with OUT.open("w", newline="") as fh:
        fh.write(f"# generated by scripts/ranged-expected.py from {INPUTS}; one row a case and a total, a "
                 f"total a case does not list being lacked; on an `undecided` or `unruled` row `low`, `high` "
                 f"and `avg` are the subtotal of what was readable, never the answer\n")
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(HEADER)
        w.writerows(rows_out)
    with OUT_COPY.open("w", newline="") as fh:
        fh.write(f"# generated by scripts/ranged-expected.py from {INPUTS}; every live pc item of the copy: "
                 f"`total` rows, the function's answers summed; `lines` rows, the lines a row names by "
                 f"template, side, array and flags beside the captured cells of the same four\n")
        w = csv.writer(fh, lineterminator="\n")
        w.writerow(["section", "total_or_template", "status_or_side", "cause_or_array", "flags", "items",
                    "lines", "sum_low", "sum_high", "sum_avg", "captured_cells", "captured_items",
                    "captured_agreeing", "asked", "rests_on", "a_sitting_would_ask"])
        w.writerows(copy_rows)
        w.writerows(group_rows)
    print(f"\nwrote {OUT.relative_to(ROOT)} ({len(rows_out)} rows), {OUT_COPY.relative_to(ROOT)} "
          f"({len(copy_rows)} `total` rows, {len(group_rows)} `lines` rows)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
