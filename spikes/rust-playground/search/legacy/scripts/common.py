#!/usr/bin/env python3
"""Shared reads for the legacy track: the Path of Building unique files, the two stores, CSV output.

Nothing here decides anything about the game. It parses what an input holds and hands it on.

The store read is copied from `../item-facts/scripts/census.py` (dedupe by GGG item id, newer
fetch wins, socketed gems lifted to their own rows) with the 2026-09-18 backup filenames.
"""

import csv
import datetime as dt
import glob
import hashlib
import json
import math
import os
import re
import sqlite3
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
TRACK = os.path.dirname(HERE)
RAW = os.path.join(TRACK, "raw")
DATA = os.path.join(TRACK, "data")

SPIKE = "spike-GERWARIC_7694-2026-09-18.db"
CPP = "cpp-userstore-GERWARIC_7694-2026-09-18.db"

# `search/legacy` → `search` → `rust-playground` → `spikes` → `acquisition` → the clones' parent.
POB = os.path.abspath(os.path.join(TRACK, "..", "..", "..", "..", "..", "PathOfBuilding"))
POB_COMMIT = "16de4b82d57f1c0de6eb40f37143c32d4da36a02"
UNIQUES = os.path.join(POB, "src", "Data", "Uniques")


def require_pob_commit():
    """Refuse to run against another checkout of the clone (the brief's rule)."""
    head = subprocess.run(["git", "-C", POB, "rev-parse", "HEAD"], capture_output=True, text=True, check=True)
    got = head.stdout.strip()
    if got != POB_COMMIT:
        sys.exit(f"PathOfBuilding is at {got}, not {POB_COMMIT}: refusing to run.")
    return got


# ---------------------------------------------------------------- Path of Building unique files

BLOCK = re.compile(r"\[\[(.*?)\]\]", re.S)

# Spec names Path of Building's own parser recognises as header lines, read from
# `src/Classes/Item.lua` (ParseRaw, lines 660-860) at POB_COMMIT. Anything else of the shape
# `Name: value` is a mod line there, and is one here.
HEADER_SPECS = {
    "Unique ID", "Item Level", "Memory Strands", "Requires Class", "Class", "Quality", "Sockets",
    "Radius", "Limited to", "Version", "Variant", "Talisman Tier", "Armour", "Evasion Rating",
    "Evasion", "Energy Shield", "Ward", "Requires Level", "Level", "LevelReq", "Has Alt Variant",
    "Has Alt Variant Two", "Has Alt Variant Three", "Has Alt Variant Four", "Has Alt Variant Five",
    "Selected Version", "Selected Variant Group", "Selected Variant", "Selected Alt Variant",
    "Selected Alt Variant Two", "Selected Alt Variant Three", "Selected Alt Variant Four",
    "Selected Alt Variant Five", "Allow Duplicate Variants", "Has Variants", "Selected Variants",
    "League", "Crafted", "Scourge", "Crucible", "Implicit", "Prefix", "Suffix", "Implicits",
    "Unreleased", "Upgrade", "Source", "Cluster Jewel Skill", "Cluster Jewel Node Count",
    "Catalyst", "CatalystQuality", "Intangibility", "Note", "Str", "Strength", "Dex", "Dexterity",
    "Int", "Intelligence", "Critical Strike Range", "Attacks per Second", "Weapon Range",
    "Critical Strike Chance", "Physical Damage", "Elemental Damage", "Chaos Damage",
    "Chance to Block", "Block chance",
}
# Lines with no value that Item.lua's ParseRaw consumes before the spec test (lines 535-561).
HEADER_BARE = {
    "--------", "Split", "Mirrored", "Corrupted", "Fractured Item", "Synthesised Item",
    "Requirements:", "Prefixes:",
    "Shaper Item", "Elder Item", "Warlord Item", "Hunter Item", "Crusader Item", "Redeemer Item",
    "Searing Exarch Item", "Eater of Worlds Item",
}
SPEC = re.compile(r"^([A-Za-z ()]+:?): (.+)$")
SPEC_REQ = re.compile(r"^(Requires [A-Za-z]+) (.+)$")
CATALYST_SPEC = re.compile(r"^Quality \([A-Za-z ]+ Modifiers\)$")
TAG = re.compile(r"\{([a-zA-Z]*):?([^}]*)\}")


def split_spec(line):
    """(spec name, value) when Path of Building would read the line as a header, else (None, None)."""
    m = SPEC.match(line)
    if not m:
        m = SPEC_REQ.match(line)
        if not m:
            return None, None
        return m.group(1), m.group(2)
    name = m.group(1).rstrip(":")
    if name in HEADER_SPECS or CATALYST_SPEC.match(name) or name.endswith("BasePercentile"):
        return name, m.group(2)
    return None, None


LEADING_TAG = re.compile(r"^\{([a-zA-Z]*):?([^}]*)\}")


def strip_tags(line):
    """Line text with its leading `{…}` tags removed; returns (text, {tag: value}).

    Only leading tags: a `Source:` header's value carries `unique{The Elder}`, which is prose and
    stays. Path of Building writes a mod line's tags at its head (`{variant:1}{tags:defences}…`).
    """
    tags = {}
    while True:
        m = LEADING_TAG.match(line)
        if not m:
            return line.strip(), tags
        tags.setdefault(m.group(1), m.group(2))
        line = line[m.end():]


class Entry:
    __slots__ = ("file", "index", "name", "base", "labels", "headers", "mods", "refusal")

    def __init__(self, **kw):
        for k in self.__slots__:
            setattr(self, k, kw.get(k))


PREAMBLE = re.compile(r"^(Item Class|Rarity): ")


def parse_entry(path, index, text, strict=True):
    """One `[[ … ]]` block. `mods` is a list of (text, variant set or None, tags).

    `strict` requires the base line to be a name `Data/Bases/*.lua` declares; the caller retries
    without it, and the entry then carries the header `base not in Data/Bases`.
    """
    if path == "Special/Generated.lua":
        return Entry(file=path, index=index,
                     refusal="Generated.lua is Lua code: its `[[ ]]` blocks are fragments of the "
                             "strings the code concatenates, not entries")
    lines = [l.strip() for l in text.replace("\r", "").split("\n")]
    lines = [l for l in lines if l]
    while lines and PREAMBLE.match(lines[0]):  # a block written in the full item-text format
        lines.pop(0)
    if len(lines) < 2:
        return Entry(file=path, index=index, refusal="fewer than two lines")
    name = lines[0]
    if split_spec(name)[0] or "{" in name:
        return Entry(file=path, index=index, refusal="the first line is not a name")
    # After the name come the base lines: one, or one per variant (`{variant:1,2}Two-Point Arrow
    # Quiver`). A header may sit between them and the name, and a header may itself be variant
    # tagged, so tags come off before the line is read. A base is a name `Data/Bases/*.lua` declares.
    labels, headers, mods, bases = [], {}, [], []
    in_bases = True
    for line in lines[1:]:
        body, tags = strip_tags(line)
        if not body:
            continue
        spec, val = split_spec(body)
        if spec == "Variant":
            labels.append(val)
            continue
        if spec:
            headers.setdefault(spec, []).append(val)
            continue
        if body in HEADER_BARE or body.endswith(" Foil Unique"):
            headers.setdefault(body, []).append("")
            continue
        if in_bases and (body in base_names() if strict else True):
            bases.append(body)
            if "variant" not in tags:
                in_bases = False
            continue
        in_bases = False
        variants = None
        if "variant" in tags:
            variants = {int(n) for n in re.findall(r"\d+", tags["variant"])}
        mods.append((body, variants, tags))
    if not bases:
        if strict:
            e = parse_entry(path, index, text, strict=False)
            if e.refusal is None:
                e.headers.setdefault("base not in Data/Bases", []).append(e.base)
            return e
        return Entry(file=path, index=index, name=name, refusal="no line after the name is a base")
    base = " / ".join(dict.fromkeys(bases))
    if not labels:
        labels = ["only"]
    over = {v for _, vs, _ in mods if vs for v in vs if v > len(labels) or v < 1}
    if over:
        return Entry(file=path, index=index, name=name, base=base, labels=labels, headers=headers,
                     mods=mods, refusal=f"{{variant:}} references {sorted(over)} of {len(labels)} variants")
    return Entry(file=path, index=index, name=name, base=base, labels=labels, headers=headers, mods=mods,
                 refusal=None)


BASES = os.path.join(POB, "src", "Data", "Bases")
BASE_KEY = re.compile(r'^itemBases\["([^"]+)"\]', re.M)
_base_names = None


def base_names():
    """Every base name Path of Building's own `Data/Bases/*.lua` declares."""
    global _base_names
    if _base_names is None:
        _base_names = set()
        for p in sorted(glob.glob(os.path.join(BASES, "*.lua"))):
            _base_names |= set(BASE_KEY.findall(open(p, encoding="utf-8").read().replace("\r", "")))
    return _base_names


def unique_files():
    """The entry files of the clone, in read order. Paths relative to `UNIQUES`."""
    top = sorted(os.path.basename(p) for p in glob.glob(os.path.join(UNIQUES, "*.lua")))
    spc = sorted("Special/" + os.path.basename(p) for p in glob.glob(os.path.join(UNIQUES, "Special", "*.lua")))
    return top + spc


def read_entries():
    """Every entry of every unique file, with its refusals."""
    out = []
    for rel in unique_files():
        text = open(os.path.join(UNIQUES, rel), encoding="utf-8").read()
        for i, block in enumerate(BLOCK.findall(text)):
            out.append(parse_entry(rel, i, block))
    return out


def variant_lines(entry, vi):
    """The mod lines of variant `vi` (1-based): every line with no `{variant:}`, plus those naming it."""
    return [t for t, vs, _ in entry.mods if vs is None or vi in vs]


def variant_tagged_lines(entry, vi):
    """As `variant_lines`, but (text, tags) — the `{tags:…}` prefix the catalyst rescale reads."""
    return [(t, tg) for t, vs, tg in entry.mods if vs is None or vi in vs]


# ---------------------------------------------------------------- selections
#
# Path of Building selects one variant, plus one more for each `Has Alt Variant…` header the entry
# carries: `Item.lua` reads the headers at lines 752-762 and the choices at 774-783, and a mod line
# is shown when any chosen variant lists it — `CheckModLineVariant`, lines 2139-2145, where an
# untagged line (`not modLine.variantList`) is always shown. The choices are independent, so the
# same variant may be chosen twice; `GetModLineVariantCount` (2148-2164) counts such a line twice
# only under `Allow Duplicate Variants`, which no entry here carries, and a repeated choice
# therefore shows exactly the lines of the distinct variants chosen. A selection is read here as
# that set: a non-empty set of at most `selection_size` variants.


def selection_size(entry):
    """How many variants Path of Building lets be chosen at once: one, plus each alt-variant header."""
    return 1 + sum(1 for h in entry.headers if h.startswith("Has Alt Variant"))


def selections(entry):
    """Every selection of the entry, as a tuple of 1-based variant indices, smallest first."""
    import itertools
    n = len(entry.labels)
    k = min(selection_size(entry), n)
    out = []
    for size in range(1, k + 1):
        out.extend(itertools.combinations(range(1, n + 1), size))
    return out


def selection_lines(entry, sel):
    """The (text, tags) lines a selection shows: every line any chosen variant lists, plus the
    untagged ones. A line the entry writes twice stays twice; the fit matches by cover."""
    return [(t, tg) for t, vs, tg in entry.mods if vs is None or any(vi in vs for vi in sel)]


# ---------------------------------------------------------------- numbers

# A Path of Building range `(a-b)`, or a plain number, with the sign written against it. A bound may
# be negative (`+(-25-50)% to Fire Resistance`), the file writes some ranges high to low
# (`(100-50)% increased Charges per use`), and the sign may sit outside the parentheses
# (`-(8-4) to Total Mana Cost of Skills`), so the sign is folded into the value and the bounds are
# sorted. That way a displayed `-24%` and a written `+(-25-50)%` reach the same template, `#%`.
# A `-` straight after a digit is a range dash, not a sign (`59-88`, item-facts census.py), and
# stays as text.
TOKEN = re.compile(r"([-+]?)\((-?\d+(?:\.\d+)?)-(-?\d+(?:\.\d+)?)\)|([-+]?)(\d+(?:\.\d+)?)")


def tokenize(text):
    """(template with every number or range as `#`, [spec, …]); spec is (lo, hi), lo == hi when fixed."""
    specs = []

    def take(m):
        sign = m.group(1) or m.group(4) or ""
        dash = sign == "-" and m.start() > 0 and text[m.start() - 1].isdigit()
        mul = -1.0 if (sign == "-" and not dash) else 1.0
        if m.group(2) is not None:
            a, b = mul * float(m.group(2)), mul * float(m.group(3))
        else:
            a = b = mul * float(m.group(5))
        specs.append((min(a, b), max(a, b)))
        return ("-#" if dash else "#")

    return TOKEN.sub(take, text), specs


def numbers_fit(item_specs, pob_specs):
    """Every item number inside the variant line's range at the same position."""
    if len(item_specs) != len(pob_specs):
        return False
    return all(lo - 1e-9 <= v <= hi + 1e-9 for (v, _), (lo, hi) in zip(item_specs, pob_specs))


RAW_TOKEN = re.compile(r"([-+]?)\((-?\d+(?:\.\d+)?)-(-?\d+(?:\.\d+)?)\)|([-+]?)(\d+(?:\.\d+)?)")


def raw_numbers(text):
    """The literal each token would be written as, in `tokenize` order; None for a range.

    Path of Building substitutes a value back into the stripped line as it stood there, with a `+`
    dropped and a `-` kept (`applyRange`, ItemTools.lua 101-112), so that is what is kept here.
    """
    out = []
    for m in RAW_TOKEN.finditer(text):
        if m.group(2) is not None:
            out.append(None)
        else:
            sign = m.group(4) or ""
            dash = sign == "-" and m.start() > 0 and text[m.start() - 1].isdigit()
            out.append(("-" if (sign == "-" and not dash) else "") + m.group(5))
    return out


# ---------------------------------------------------------------- the catalyst rescale
#
# `src/Classes/Item.lua` lines 14-29 at POB_COMMIT, copied. Index i is one catalyst across the
# three lists: the currency's name, the descriptor `ParseRaw` matches a `Quality (<descriptor>
# Modifiers)` property against (lines 673-677), and the mod tags it scales.

CATALYST_LIST = ["Abrasive", "Accelerating", "Dextral", "Fertile", "Imbued", "Intrinsic",
                 "Noxious", "Prismatic", "Sinistral", "Tempering", "Turbulent", "Unstable"]
CATALYST_DESCRIPTORS = ["Attack", "Speed", "Suffix", "Life and Mana", "Caster", "Attribute",
                        "Physical and Chaos", "Resistance", "Prefix", "Defence", "Elemental",
                        "Critical"]
CATALYST_TAGS = [
    ["attack"],
    ["speed"],
    ["suffix"],
    ["life", "mana", "resource"],
    ["caster"],
    ["jewellery_attribute", "attribute"],
    ["physical_damage", "chaos_damage"],
    ["jewellery_resistance", "resistance"],
    ["prefix"],
    ["jewellery_defense", "defences", "armour", "evasion", "energyshield"],
    ["jewellery_elemental", "elemental_damage"],
    ["critical"],
]

# The items spell the kind as the game does; `catalystDescriptorList` spells two of them shorter,
# so the map is by position: what an item's property shows on the left, the index above on the
# right. `Elemental Damage` and `Physical and Chaos Damage` are the two that differ, and a
# `Quality (Elemental Damage Modifiers)` property therefore sets no catalyst in Path of Building
# itself (its test is equality, line 675).
DESCRIPTOR_INDEX = {
    "Attack": 0, "Speed": 1, "Suffix": 2, "Life and Mana": 3, "Caster": 4, "Attribute": 5,
    "Physical and Chaos Damage": 6, "Resistance": 7, "Prefix": 8, "Defence": 9,
    "Elemental Damage": 10, "Critical": 11,
}
# Sinistral and Dextral key on `prefix` and `suffix`, which `getCatalystScalar` takes from a mod
# line's own flag rather than from `{tags:…}` (lines 48-53). No line of any unique file read here
# carries either flag, so an item of those two kinds has nothing to match on and is counted apart.
PREFIX_SUFFIX_INDEX = {2, 8}

TAG_WORD = re.compile(r"[A-Za-z_]+")


def catalyst_index(descriptor):
    """The catalyst index an item's `Quality (<kind> Modifiers)` names, or None."""
    return DESCRIPTOR_INDEX.get(descriptor)


def line_scalar(text, tags, index, quality):
    """`getCatalystScalar` (Item.lua 31-62) over a unique file's mod line: `(100+q)/100`, or 1.

    1 when the line is marked unscalable (lines 32-34, and the ` - Unscalable Value` suffix
    ParseRaw strips at 1083-1085), when the line carries no `{tags:…}` (line 36), or when none of
    its tags is one the catalyst scales.
    """
    if index is None:
        return 1.0
    if "unscalable" in tags or text.endswith(" - Unscalable Value"):
        return 1.0
    line_tags = set(TAG_WORD.findall(tags.get("tags", "")))
    if not line_tags:
        return 1.0
    if not line_tags & set(CATALYST_TAGS[index]):
        return 1.0
    return (100 + (20 if quality is None else quality)) / 100


# `data.modScalability` — `src/Data/ModScalability.lua`, loaded by `src/Modules/Data.lua` line 435.
# Keyed by the mod line with every number replaced by `#`, it says per remaining `#` whether the
# value scales and how the game formats it.
MOD_SCALABILITY = os.path.join(POB, "src", "Data", "ModScalability.lua")
MS_ROW = re.compile(r'^\t\["((?:[^"\\]|\\.)*)"\] = \{(.*)\},$')
MS_ENTRY = re.compile(r'\{ isScalable = (true|false)(?:, formats = \{ ((?:"[^"]*"(?:, )?)+) \})? \}')
# `applyRange`, ItemTools.lua 195-286: a format sets `precision` (a multiplier into the value the
# game stores) and `displayPrecision` (decimals shown); the last format of a list wins.
FORMAT_PRECISION = {
    "divide_by_two_0dp": (2, 0), "divide_by_three": (3, None), "divide_by_four": (4, None),
    "divide_by_five": (5, None), "divide_by_six": (6, None), "divide_by_ten_0dp": (10, 0),
    "divide_by_ten_1dp": (10, 1), "divide_by_ten_1dp_if_required": (10, 1),
    "divide_by_twelve": (12, None), "divide_by_fifteen_0dp": (15, 0), "divide_by_twenty": (20, None),
    "divide_by_twenty_then_double_0dp": (10, 0), "divide_by_one_hundred": (100, None),
    "divide_by_one_hundred_and_negate": (100, None), "divide_by_one_hundred_0dp": (100, 0),
    "divide_by_one_hundred_1dp": (100, 1), "divide_by_one_hundred_2dp": (100, 2),
    "divide_by_one_hundred_2dp_if_required": (100, 2), "divide_by_one_thousand": (1000, None),
    "divide_by_ten_thousand_1dp": (10000, 1), "per_minute_to_per_second": (60, None),
    "per_minute_to_per_second_0dp": (60, 0), "per_minute_to_per_second_1dp": (60, 1),
    "per_minute_to_per_second_2dp": (60, 2), "per_minute_to_per_second_2dp_if_required": (60, 2),
    "milliseconds_to_seconds": (1000, None), "milliseconds_to_seconds_halved": (1000, None),
    "milliseconds_to_seconds_0dp": (1000, 0), "milliseconds_to_seconds_1dp": (1000, 1),
    "milliseconds_to_seconds_2dp": (1000, 2), "milliseconds_to_seconds_2dp_if_required": (1000, 2),
    "locations_to_metres": (10, 1), "deciseconds_to_seconds": (10, None),
}
DEFAULT_HIGH_PRECISION = 1  # `data.defaultHighPrecision`, Data.lua line 434
_ms = None


def mod_scalability():
    """{line with every number as `#`: [(scales, precision, display precision), …]}."""
    global _ms
    if _ms is None:
        _ms = {}
        with open(MOD_SCALABILITY, encoding="utf-8", newline="\n") as fh:
            for raw in fh:
                m = MS_ROW.match(raw.replace("\r", "").rstrip("\n"))
                if not m:
                    continue
                spec = []
                for scalable, formats in MS_ENTRY.findall(m.group(2)):
                    prec, disp = 1, None
                    for name in re.findall(r'"([^"]*)"', formats or ""):
                        if name in FORMAT_PRECISION:
                            prec, disp = FORMAT_PRECISION[name]
                    spec.append((scalable == "true", prec, disp))
                _ms[m.group(1)] = spec
    return _ms


def _round_sym(v, dec=None):
    """`roundSymmetric`, Common.lua 733-751: round half away from zero, at `dec` decimals."""
    if dec is None:
        return math.floor(v + 0.5) if v >= 0 else math.ceil(v - 0.5)
    f = 10.0 ** dec
    return (math.floor(v * f + 0.5) if v >= 0 else math.ceil(v * f - 0.5)) / f


def format_value(value, scalar, precision, display_precision):
    """`itemLib.formatValue` (ItemTools.lua 61-75) with no corrupted multiplier: into the stored
    value, floor toward zero against the scalar (`floorSymmetric`, Common.lua 766-773), back."""
    v = _round_sym(value * precision)
    if scalar != 1:
        v = math.trunc(v * scalar)
    v = v / precision
    if display_precision is not None:
        return _round_sym(v, display_precision)
    return _round_sym(v, min(2, math.floor(math.log10(precision) + 0.001)))


def _substitute(template, values):
    """The template with the `#` at each given position replaced by its literal."""
    parts = template.split("#")
    out = [parts[0]]
    for i, part in enumerate(parts[1:]):
        out.append(values.get(i, "#"))
        out.append(part)
    return "".join(out)


def scalability_plan(text, specs):
    """Per number: (scales, precision, display precision), as `findScalableLine` decides it.

    `findScalableLine` (ItemTools.lua 121-186) substitutes values back into the stripped line, the
    most first, and takes the first key `data.modScalability` holds; only a value written as a
    fixed number can match a key's literal, so only those are substituted here. With no key, the
    old method runs (lines 301-355): the first *n* numbers scale, *n* being the count of `(a-b)`
    ranges or 1, at one decimal when the line writes one.
    """
    import itertools
    template, _ = tokenize(text)
    raws = raw_numbers(text)
    ms = mod_scalability()
    fixed = [i for i, r in enumerate(raws) if r is not None]
    for size in range(len(fixed), 0, -1):
        for combo in itertools.combinations(fixed, size):
            spec = ms.get(_substitute(template, {i: raws[i] for i in combo}))
            if spec is None:
                continue
            rest = [i for i in range(len(specs)) if i not in combo]
            plan = [(False, 1, None)] * len(specs)
            for pos, s in zip(rest, spec):
                plan[pos] = s
            return plan
    spec = ms.get(template)
    if spec is not None:
        plan = [(False, 1, None)] * len(specs)
        for pos, s in enumerate(spec):
            if pos < len(plan):
                plan[pos] = s
        return plan
    n = sum(1 for lo, hi in specs if lo != hi) or (1 if specs else 0)
    dec = DEFAULT_HIGH_PRECISION if re.search(r"\d+\.\d", text) else 0
    return [(i < n, -(10 ** dec), None) for i in range(len(specs))]


def scale_line(text, tags, index, quality):
    """(template, specs, scaled) for a variant line under an item's catalyst.

    A negative precision marks the fallback path, whose arithmetic is `applyValueScalar`'s
    (ItemTools.lua 37-57) rather than `formatValue`'s.
    """
    template, specs = tokenize(text)
    scalar = line_scalar(text, tags, index, quality)
    if scalar == 1.0 or not specs:
        return template, specs, False
    out = []
    for (lo, hi), (scales, precision, display) in zip(specs, scalability_plan(text, specs)):
        if not scales:
            out.append((lo, hi))
        elif precision < 0:
            power = -precision
            nudge = 0.001 if power == 1 else 0
            out.append(tuple(math.floor(v * scalar * power + nudge) / power for v in (lo, hi)))
        else:
            out.append((format_value(lo, scalar, precision, display),
                        format_value(hi, scalar, precision, display)))
    return template, out, out != specs


# ---------------------------------------------------------------- the stores


class Rec:
    __slots__ = ("id", "source", "realm", "league", "kind", "container", "socketed_in", "fetched", "json")

    def __init__(self, **kw):
        for k in self.__slots__:
            setattr(self, k, kw.get(k))


def _is_item(v):
    return isinstance(v, dict) and "typeLine" in v


def _item_arrays(body):
    return [k for k, v in body.items() if isinstance(v, list) and v and all(_is_item(e) for e in v)]


def _lift(body, realm, league, kind, fetched):
    for container in _item_arrays(body):
        for it in body[container]:
            gems = it.pop("socketedItems", None)
            yield Rec(id=it.get("id"), source="cpp", realm=realm, league=league, kind=kind,
                      container=container, socketed_in=None, fetched=fetched, json=it)
            for g in gems or []:
                if "id" not in g:
                    continue
                yield Rec(id=g["id"], source="cpp", realm=realm, league=league, kind=kind,
                          container=container, socketed_in=it.get("id"), fetched=fetched, json=g)


def read_items():
    """Every item both stores hold, deduplicated by GGG item id; the newer fetch wins."""
    recs = []
    db = sqlite3.connect(f"file:{os.path.join(RAW, SPIKE)}?mode=ro", uri=True)
    q = """SELECT i.id, i.realm, i.league, i.location_kind, i.container, i.socketed_in, i.last_seen, i.json
           FROM items i WHERE i.removed_at IS NULL"""
    for r in db.execute(q):
        recs.append(Rec(id=r[0], source="spike", realm=r[1], league=r[2], kind=r[3], container=r[4],
                        socketed_in=r[5], fetched=r[6], json=json.loads(r[7])))
    db2 = sqlite3.connect(f"file:{os.path.join(RAW, CPP)}?mode=ro", uri=True)
    for realm, league, fetched_at, body in db2.execute(
            "SELECT realm, league, json_fetched_at, json_data FROM stashes WHERE json_data IS NOT NULL"):
        recs.extend(_lift(json.loads(body), realm, league, "stash",
                          int(dt.datetime.fromisoformat(fetched_at).timestamp())))
    for realm, league, fetched_at, body in db2.execute(
            "SELECT realm, league, json_fetched_at, json_data FROM characters WHERE json_data IS NOT NULL"):
        recs.extend(_lift(json.loads(body), realm, league, "character",
                          int(dt.datetime.fromisoformat(fetched_at).timestamp())))
    by_id = {}
    for rec in recs:
        old = by_id.get(rec.id)
        if old is None or rec.fetched > old.fetched:
            by_id[rec.id] = rec
    return list(by_id.values())


def mod_arrays(item):
    """(array name, [display line, …], [flag string, …]) for every `*Mods` array the item carries.

    A line an array gives as an object carries its text in `description`; an `ultimatumMods`
    element carries `type` instead (item-facts census.py). A line holding a newline is split,
    because a Path of Building entry writes each such line on its own.
    """
    for k, v in item.items():
        if not k.endswith("Mods") or not isinstance(v, list):
            continue
        texts, flags = [], []
        for line in v:
            if isinstance(line, dict):
                text = line.get("description", line.get("type", ""))
                for fk, fv in (line.get("flags") or {}).items():
                    if fv:
                        flags.append(fk)
            else:
                text = str(line)
            for part in text.replace("\r", "").split("\n"):
                if part.strip():
                    texts.append(part.strip())
        yield k, texts, flags


CATALYST_PROP = re.compile(r"^Quality \(([A-Za-z ]+) Modifiers\)$")


def catalyst_quality(item):
    """The catalyst descriptor and quality when a `properties` row names a modifier kind.

    The rule is Path of Building's own: `src/Classes/Item.lua` ParseRaw reads a spec whose name
    matches `Quality %([%a%s]+ Modifiers%)` as `catalyst` + `catalystQuality`, against
    `catalystDescriptorList` (line 15). `Quality (Quantity)` on a map does not match it.
    """
    for p in item.get("properties") or []:
        if not isinstance(p, dict):
            continue
        m = CATALYST_PROP.match(p.get("name", "") or "")
        if not m:
            continue
        vals = p.get("values") or []
        raw = vals[0][0] if vals and isinstance(vals[0], list) and vals[0] else ""
        n = re.search(r"(\d+)", str(raw))
        return m.group(1), int(n.group(1)) if n else None
    return None, None


ICON_ART = re.compile(r"/gen/image/([A-Za-z0-9_=-]+)/")


def icon_dir(url):
    """The art path inside a poecdn icon URL (copied from item-facts census.py)."""
    import base64
    f = None
    if "/gen/image/" in url:
        token = url.split("/gen/image/")[1].split("/")[0]
        token += "=" * (-len(token) % 4)
        try:
            spec = json.loads(base64.urlsafe_b64decode(token))
            for part in spec:
                if isinstance(part, dict) and "f" in part:
                    f = part["f"]
        except Exception:
            return "?"
    elif "/image/Art/" in url:
        f = url.split("/image/Art/")[1]
    if not f:
        return "?"
    segs = f.split("/")
    if segs and segs[0] == "2DItems":
        segs = segs[1:]
    return "/".join(segs[:2])


def row_key(item_id):
    """A stable row key that is not the GGG item id (the brief's scrub guard)."""
    return hashlib.sha256(("legacy:" + item_id).encode()).hexdigest()[:12]


# ---------------------------------------------------------------- output

ACCOUNT = re.compile(r"GERWARIC|_vagabond_6960|[#_]7694", re.I)
GGG_ID = re.compile(r"\b[0-9a-f]{32,}\b")


def write_csv(name, header, rows):
    """Strict CSV, `\\n` endings, a header first line and no comment line; scrubbed before it lands."""
    os.makedirs(DATA, exist_ok=True)
    path = os.path.join(DATA, name)
    from io import StringIO
    buf = StringIO()
    w = csv.writer(buf, lineterminator="\n")
    w.writerow(header)
    for r in rows:
        w.writerow(r)
    text = buf.getvalue()
    for pat, what in ((ACCOUNT, "an account name"), (GGG_ID, "a full GGG item id")):
        m = pat.search(text)
        if m:
            sys.exit(f"scrub guard: {name} would carry {what} ({m.group(0)!r}); refusing to write.")
    open(path, "w", newline="").write(text)
    print(f"  wrote data/{name}: {len(rows)} rows, {len(text.encode())} bytes")
