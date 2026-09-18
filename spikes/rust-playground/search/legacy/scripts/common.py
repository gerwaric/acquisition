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
