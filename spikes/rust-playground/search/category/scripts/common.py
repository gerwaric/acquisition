"""category: what the three scripts share — the pinned inputs, the readings, the copy.

Not run on its own. Every input is read-only; a clone at another commit is refused, as
search/repoe/scripts/base-taxonomy.py refuses (a pull is a new manifest row first).

The export's class -> trade id map and its name rules are not restated here: they are imported
from search/repoe/scripts/base-taxonomy.py (CLASS_TO_TRADE, NAME_RULES, PARENTS, predict), so the
rule lives in one place and this track measures it.
"""

import collections
import csv
import importlib.util
import json
import os
import re
import sqlite3
import subprocess
import sys
import tomllib

HERE = os.path.dirname(os.path.abspath(__file__))
TRACK = os.path.dirname(HERE)
SEARCH = os.path.dirname(TRACK)
ROOT = os.path.dirname(SEARCH)
CLONES = os.path.dirname(os.path.dirname(os.path.dirname(ROOT)))  # siblings of acquisition/
DATA = os.path.join(TRACK, "data")

POE1 = os.path.join(CLONES, "poe1")
POE1_COMMIT = "e2bd511a0133bbe6c1ab548ef1285cb99f3cf0e9"  # search/repoe/MANIFEST.md
POB = os.path.join(CLONES, "PathOfBuilding")
POB_COMMIT = "16de4b82d57f1c0de6eb40f37143c32d4da36a02"  # search/repoe/MANIFEST.md
APT = os.path.join(CLONES, "awakened-poe-trade")
APT_COMMIT = "ce551eb7a9b704fbdcc2478eebb26be8f91786c7"  # search/prior-art/MANIFEST.md

TQ = os.path.join(SEARCH, "trade-query", "data")
TQ_FETCHES = os.path.join(SEARCH, "trade-query", "raw", "searches")
COPY = os.path.join(SEARCH, "item-facts", "raw", "m3", "store", "mock", "GERWARIC_7694.db")
TABLE = os.path.join(ROOT, "crates", "acquisition-search", "reference", "classes-v1.toml")
TAXONOMY = os.path.join(SEARCH, "repoe", "scripts", "base-taxonomy.py")

# the three constants the class table's reader uses (crates/acquisition-search/src/class.rs, lines 113-121)
MAP_PREFIXES = ["Blighted ", "Blight-ravaged "]
GEM_CLASSES = {"Skill Gems", "Support Gems"}
QUEST_CLASS = "Quest Items"

BEASTS = ["monster.beast", "monster.yellowbeast", "monster.redbeast"]
BEAST_HELP = "Right-click to add this to your bestiary."  # the body's descrText; APT's BEAST_HELP (client_strings.js line 57)


def pinned(path, commit):
    head = subprocess.run(["git", "-C", path, "rev-parse", "HEAD"], capture_output=True, text=True, check=True).stdout.strip()
    if head != commit:
        sys.exit(f"{path} is at {head}, not the pinned {commit}: a pull is a new manifest row before re-running")


def taxonomy():
    spec = importlib.util.spec_from_file_location("base_taxonomy", TAXONOMY)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def write_csv(name, header_line, columns, rows):
    os.makedirs(DATA, exist_ok=True)
    with open(os.path.join(DATA, name), "w", newline="") as f:
        f.write(header_line + "\n")
        w = csv.writer(f, lineterminator="\n")
        w.writerow(columns)
        w.writerows(rows)


def site_filters():
    """[(id, text)] in the filter's order, the empty 'Any' option dropped."""
    filters = json.load(open(os.path.join(TQ, "filters-2026-09-12.json")))["result"]
    return [(o["id"], o["text"]) for g in filters for f in g["filters"] if f["id"] == "category"
            for o in f["option"]["options"] if o["id"]]


def site_coarse():
    """base type -> set of coarse category ids (items-2026-09-12.json), and the coarse ids with labels."""
    items = json.load(open(os.path.join(TQ, "items-2026-09-12.json")))["result"]
    by_type = collections.defaultdict(set)
    for c in items:
        for e in c["entries"]:
            by_type[e["type"]].add(c["id"])
    return by_type, [(c["id"], c["label"], len(c["entries"])) for c in items]


def structure(ids):
    """The site's ids as its data shapes them: a prefix parent has another id under `<it>.`;
    a grouping is an id whose members no dot names (weapon.one, weapon.dagger, ...): an id with a
    dot whose own prefix is a parent and that is not a leaf of the export. Returned: kind per id."""
    idset = set(ids)
    prefix_parents = {i for i in ids if any(j.startswith(i + ".") for j in ids)}
    return prefix_parents, idset


class Export:
    """The pinned export: base name -> internal classes (every release state, as the class table
    reads it), transfigured gems by display name (rule 2 of tools/class-table.py), and the classes."""

    def __init__(self):
        pinned(POE1, POE1_COMMIT)
        d = os.path.join(POE1, "data")
        self.bases = json.load(open(os.path.join(d, "base_items.json")))
        self.classes = json.load(open(os.path.join(d, "item_classes.json")))
        gems = json.load(open(os.path.join(d, "gems.json")))
        self.mods = json.load(open(os.path.join(d, "mods.json")))
        self.by_name = collections.defaultdict(set)
        for b in self.bases.values():
            if b["name"]:
                self.by_name[b["name"]].add(b["item_class"])
        for g in gems.values():
            bi = g.get("base_item")
            if not bi:
                continue
            name = (g.get("active_skill") or {}).get("display_name") or bi["display_name"]
            if name:
                self.by_name[name].add(self.bases[bi["id"]]["item_class"])

    def display(self, cls):
        return self.classes[cls]["name"] or ""


def load_table():
    t = tomllib.load(open(TABLE, "rb"))
    by_base = collections.defaultdict(set)
    for c in t["class"]:
        for b in c["bases"]:
            by_base[b].add(c["name"])
    return t, by_base


def shown(raw):
    """derive::shown (crates/acquisition-search/src/derive.rs, line 1150): `<style>{X}` -> X, `[A|B]` -> B."""
    text = (raw or "").replace("\r\n", "\n").replace("\r", "\n")
    out, i, open_ = [], 0, 0
    while i < len(text):
        c = text[i]
        if c == "<":
            end = text.find(">", i)
            if end >= 0 and text[end + 1:end + 2] == "{":
                open_ += 1
                i = end + 2
                continue
        if c == "}" and open_ > 0:
            open_ -= 1
        else:
            out.append(c)
        i += 1
    s = "".join(out)
    return re.sub(r"\[([^\]]*)\]", lambda m: m.group(1).rsplit("|", 1)[-1], s)


def copy_items():
    """Every live pc item of the copy, read-only: (id, league, location_kind, json dict).
    Returns None when the copy is absent (every count that needs it is then an open question)."""
    if not os.path.exists(COPY):
        return None
    db = sqlite3.connect(f"file:{COPY}?mode=ro", uri=True)
    rows = db.execute("SELECT id, league, location_kind, json FROM items WHERE realm = 'pc' AND removed_at IS NULL ORDER BY id").fetchall()
    db.close()
    return [(i, lg, lk, json.loads(js)) for i, lg, lk, js in rows]


def is_beast(body):
    return body.get("descrText") == BEAST_HELP


def classes_of(export, base, frame):
    """The export's internal classes of a base as the class table's reader picks them
    (class.rs lines 370-460): exact name, else past a blight prefix; several -> the frame picks.
    Returns (classes, how)."""
    found = set(export.by_name.get(base, ()))
    how = "base"
    if not found:
        for p in MAP_PREFIXES:
            if base.startswith(p) and len(export.by_name.get(base[len(p):], ())) == 1:
                found, how = set(export.by_name[base[len(p):]]), "base past a blight prefix"
                break
    if len(found) > 1 and frame:
        disp = {c: export.display(c) for c in found}
        if frame.lower() == "gem":
            picked = {c for c in found if disp[c] in GEM_CLASSES}
        elif frame.lower() == "quest":
            picked = {c for c in found if disp[c] == QUEST_CLASS}
        else:
            picked = {c for c in found if disp[c] not in GEM_CLASSES and disp[c] != QUEST_CLASS}
        if picked:
            found, how = picked, how + ", frame picks"
    return found, how


def place(export, tax, body):
    """The readings on one item: the ids it is placed under, and why.
    Returns (ids, basis, classes, reason) where reason names a residue kind or ''."""
    base = shown(body.get("baseType"))
    if is_beast(body):
        return set(BEASTS), "body rule (descrText)", set(), "a beast: which of the three leaves is not settled on this machine"
    if not base:
        return set(), "", set(), "no base"
    cls, how = classes_of(export, base, body.get("frameTypeId"))
    if not cls:
        return set(), "", set(), "no base in any table"
    ids, bases_ = set(), set()
    for c in cls:
        fine, basis = tax.predict(c, base)
        if fine:
            ids.add(fine)
            bases_.add(basis)
    if not ids:
        return set(), how, cls, "a class with no id"
    if len(ids) > 1:
        return ids, how, cls, "a base under several ids"
    return ids, how + "; " + "/".join(sorted(bases_)), cls, ""


def apt():
    """Awakened PoE Trade: base name -> trade id, by items.ndjson's craftable.category through
    CATEGORY_TO_TRADE_ID (pathofexile-trade.ts) and the ItemCategory enum (parser/meta.ts).
    Returns (placements {name: id}, unmapped {name: category}, the map {category string: id}, beasts [names])."""
    pinned(APT, APT_COMMIT)
    meta = open(os.path.join(APT, "renderer", "src", "parser", "meta.ts")).read()
    enum = dict(re.findall(r"^\s+(\w+) = '([^']+)'", meta.split("export enum ItemCategory")[1].split("}")[0], re.M))
    trade_ts = open(os.path.join(APT, "renderer", "src", "web", "price-check", "trade", "pathofexile-trade.ts")).read()
    block = trade_ts.split("export const CATEGORY_TO_TRADE_ID = new Map([")[1].split("])")[0]
    cmap = {enum[k]: v for k, v in re.findall(r"\[ItemCategory\.(\w+), '([^']+)'\]", block)}
    place_, unmapped, beasts = {}, {}, []
    for line in open(os.path.join(APT, "renderer", "public", "data", "en", "items.ndjson")):
        j = json.loads(line)
        if j.get("namespace") == "CAPTURED_BEAST":
            beasts.append(j["name"])
        cat = (j.get("craftable") or {}).get("category")
        if not cat:
            continue
        if cat in cmap:
            place_[j["name"]] = cmap[cat]
        else:
            unmapped[j["name"]] = cat
    return place_, unmapped, cmap, beasts


POB_SLOT = {  # the slot a base of the type is equipped in; getTradeCategory reads the slot first
    "Body Armour": "Body Armour", "Helmet": "Helmet", "Gloves": "Gloves", "Boots": "Boots",
    "Amulet": "Amulet", "Ring": "Ring 1", "Belt": "Belt", "Flask": "Flask 1", "Jewel": "Jewel",
    "Shield": "Weapon 2", "Quiver": "Weapon 2",
}


def pob():
    """Path of Building: every base in src/Data/Bases/*.lua with its type and subType, placed by
    M.getTradeCategory (src/Classes/TradeHelpers.lua), whose branches are read from the file, not
    transcribed. Returns (placements {name: id or ''}, info {name: (type, subType, file)}, the function's line)."""
    pinned(POB, POB_COMMIT)
    info = {}
    bdir = os.path.join(POB, "src", "Data", "Bases")
    for fn in sorted(os.listdir(bdir)):
        if not fn.endswith(".lua"):
            continue
        text = open(os.path.join(bdir, fn), encoding="utf-8").read().replace("\r", "")
        for m in re.finditer(r'^itemBases\["([^"]+)"\] = \{\n(.*?)^\}', text, re.M | re.S):
            body = m.group(2)
            t = re.search(r'^\ttype = "([^"]+)"', body, re.M)
            s = re.search(r'^\tsubType = "([^"]+)"', body, re.M)
            info[m.group(1)] = (t.group(1) if t else "", s.group(1) if s else "", fn)
    lua = open(os.path.join(POB, "src", "Classes", "TradeHelpers.lua"), encoding="utf-8").read().replace("\r", "")
    lines = lua.split("\n")
    start = next(i for i, l in enumerate(lines) if l.startswith("function M.getTradeCategory("))
    end = next(i for i in range(start, len(lines)) if lines[i] == "end")
    fn_text = lines[start:end + 1]
    by_type = dict(re.findall(r'itemType == "([^"]+)" then return "([^"]+)"', "\n".join(fn_text)))
    finds = re.findall(r'itemType:find\("([^"]+)"\) then return "([^"]+)"', "\n".join(fn_text))
    slot_eq = {}
    for l in fn_text:
        m = re.search(r'return "([^"]+)"', l)
        if m and "slotName ==" in l:
            for s in re.findall(r'slotName == "([^"]+)"', l):
                slot_eq[s] = m.group(1)
    slot_find = [(s, m) for s, m in re.findall(r'slotName:find\("([^"]+)"\) then return "([^"]+)"', "\n".join(fn_text))]

    def trade(typ, sub):
        slot = "Abyssal Socket" if (typ == "Jewel" and sub == "Abyss") else POB_SLOT.get(typ, "Weapon 1" if typ not in ("Graft", "Tincture") else None)
        if slot is None:
            return ""
        if re.match(r"^Weapon \d", slot):
            if typ in by_type:
                return by_type[typ]
            for pat, tid in finds:
                if pat in typ:
                    return tid
            return "weapon"
        if slot in slot_eq:
            return slot_eq[slot]
        for pat, tid in slot_find:
            if pat in slot:
                return tid
        return ""

    return {n: trade(t, s) for n, (t, s, _) in info.items()}, info, start + 1


def site_fetches():
    """The trade-query captures (raw, local): per category option searched, the base types the
    first fetch returned, with the clipboard `Item Class:` line. {category: Counter((baseType, class))}.
    Only category, base type and class are read; nothing about a seller."""
    import base64
    import glob
    out = collections.defaultdict(collections.Counter)
    for req in sorted(glob.glob(os.path.join(TQ_FETCHES, "*-request.json"))):
        q = os.path.basename(req)[: -len("-request.json")]
        try:
            body = json.load(open(req))
        except ValueError:
            continue
        cat = (((body.get("query") or {}).get("filters") or {}).get("type_filters") or {}).get("filters", {}).get("category", {}).get("option")
        fetch = os.path.join(TQ_FETCHES, q + "-fetch.json")
        if not cat or not os.path.exists(fetch) or os.path.getsize(fetch) == 0:
            continue
        for r in json.load(open(fetch)).get("result", []):
            it = r["item"]
            t = (it.get("extended") or {}).get("text")
            klass = ""
            if t:
                first = base64.b64decode(t).decode("utf-8", "replace").split("\n")[0].rstrip("\r")
                if first.startswith("Item Class: "):
                    klass = first[len("Item Class: "):]
            out[cat][(it.get("baseType"), klass)] += 1
    return out
