#!/usr/bin/env python3
"""Compare the Rust store (raw/spike-GERWARIC_7694-2026-09-13.db) and the C++ store (raw/cpp-userstore-GERWARIC_7694-2026-09-13.db) for the
same account's Standard/pc stash items, joined by GGG item id."""
import json, sqlite3, sys, collections

import os
TRACK = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RUST = os.path.join(TRACK, "raw", "spike-GERWARIC_7694-2026-09-13.db")
CPP = os.path.join(TRACK, "raw", "cpp-userstore-GERWARIC_7694-2026-09-13.db")

def canon(o):
    return json.dumps(o, sort_keys=True, separators=(",", ":"), ensure_ascii=False)

# --- side A: rust ---
rust = {}
con = sqlite3.connect(RUST)
for (iid, js) in con.execute(
        "select id, json from items where removed_at is null and realm='pc' "
        "and league='Standard' and location_kind='stash'"):
    o = json.loads(js)
    o.pop("socketedItems", None)
    rust[iid] = o
con.close()

# --- side B: cpp, flattened ---
cpp = {}
dupes = 0
con = sqlite3.connect(CPP)
for (jd,) in con.execute(
        "select json_data from stashes where realm='pc' and league='Standard' "
        "and json_data is not null"):
    tab = json.loads(jd)
    items = tab.get("items") or []
    for it in items:
        gems = it.get("socketedItems") or []
        host = dict(it)
        host.pop("socketedItems", None)
        if host.get("id") in cpp: dupes += 1
        if host.get("id"): cpp[host["id"]] = host
        for g in gems:
            g2 = dict(g)
            g2.pop("socketedItems", None)
            if g2.get("id"): cpp[g2["id"]] = g2
con.close()

rk, ck = set(rust), set(cpp)
both = rk & ck

KEYCLASS = {
    "x": "positional/location", "y": "positional/location",
    "inventoryId": "positional/location", "league": "positional/location",
    "realm": "positional/location", "socket": "positional/location",
    "colour": "positional/location", "note": "user annotation",
    "forum_note": "user annotation", "icon": "icon",
    "properties": "properties/requirements",
    "additionalProperties": "properties/requirements",
    "nextLevelRequirements": "properties/requirements",
    "requirements": "properties/requirements", "sockets": "sockets",
}
MODKEYS = {"implicitMods","explicitMods","craftedMods","enchantMods","veiledMods",
           "utilityMods","fracturedMods","scourgeMods","crucibleMods","cosmeticMods",
           "pseudoMods","logbookMods","grantedSkills","extended","hybrid",
           "implicitModsHash","explicitModsHash"}
def klass(k):
    if k in KEYCLASS: return KEYCLASS[k]
    if k in MODKEYS or k.endswith("Mods"): return "mod array"
    return "other"

identical = 0
keydiff = collections.Counter()
modtext_examples = []
modtext_ids = set()
icon_diff = 0
props_diff = 0
ident_flip = 0
veiled_involved = 0
sample_by_key = {}

for iid in both:
    a, b = cpp[iid], rust[iid]   # a = old (C++, Jul/Aug), b = new (Rust, Sep)
    if canon(a) == canon(b):
        identical += 1
        continue
    keys = set(a) | set(b)
    for k in keys:
        if canon(a.get(k)) != canon(b.get(k)):
            keydiff[k] += 1
            sample_by_key.setdefault(k, (iid, a.get(k), b.get(k)))
            if k == "icon": icon_diff += 1
            if k == "properties": props_diff += 1
            if klass(k) == "mod array" and k not in ("extended","hybrid"):
                oldv, newv = a.get(k), b.get(k)
                if isinstance(oldv, list) or isinstance(newv, list):
                    modtext_ids.add(iid)
                    if len(modtext_examples) < 40:
                        modtext_examples.append((iid, k, a.get("name",""),
                            a.get("typeLine",""), oldv, newv, a, b))
    if a.get("identified") is False and b.get("identified") is True:
        ident_flip += 1

def is_veiled(a, b):
    for o in (a, b):
        if "veiledMods" in o or o.get("veiled") is True: return True
        for k, v in o.items():
            if isinstance(v, list) and (k.endswith("Mods")):
                for s in v:
                    if isinstance(s, str) and "Veiled" in s: return True
    return False

for e in modtext_examples:
    if is_veiled(e[6], e[7]): veiled_involved += 1

L = []
w = L.append
w("# Drift between the two captures\n")
w(f"C++ store (`raw/cpp-userstore-GERWARIC_7694-2026-09-13.db`, fetched 2026-07-20..08-13) vs Rust store "
  f"(`raw/spike-GERWARIC_7694-2026-09-13.db`, fetched 2026-09-07..11); Standard / pc / stash only.\n")
w("## Counts\n")
w(f"| side | items |")
w(f"| --- | --- |")
w(f"| Rust (`items`, live, stash) | {len(rk)} |")
w(f"| C++ (`stashes` flattened: top-level + socketed gems) | {len(ck)} |")
w(f"| ids in both | {len(both)} |")
w(f"| only in Rust (new since Aug) | {len(rk-ck)} |")
w(f"| only in C++ (gone by Sep) | {len(ck-rk)} |")
w(f"\n(duplicate top-level ids across C++ tabs: {dupes})\n")
w("## Common ids: identical vs. changed\n")
w(f"| | count | share |")
w(f"| --- | --- | --- |")
w(f"| byte-identical canonical JSON | {identical} | {identical/max(len(both),1):.1%} |")
w(f"| differ | {len(both)-identical} | {(len(both)-identical)/max(len(both),1):.1%} |")
w("\n## Top-level keys that differ\n")
w("| key | items | class | example old -> new |")
w("| --- | ---: | --- | --- |")
for k, n in keydiff.most_common():
    iid, ov, nv = sample_by_key[k]
    def short(v):
        s = canon(v) if v is not None else "(absent)"
        return (s[:70] + "…") if len(s) > 70 else s
    w(f"| `{k}` | {n} | {klass(k)} | {short(ov).replace('|','\\|')} -> {short(nv).replace('|','\\|')} |")
w(f"\n## Mod-array text\n")
w(f"items with a differing mod array: **{len(modtext_ids)}**; "
  f"unidentified->identified flips among all differing items: **{ident_flip}**\n")
if modtext_examples:
    w("\nExamples (up to 10):\n")
    for e in modtext_examples[:10]:
        iid, k, name, tl, ov, nv, _, _ = e
        w(f"- `{iid[:16]}…` **{name or '(none)'}** / {tl} — key `{k}`")
        w(f"  - old: {canon(ov)}")
        w(f"  - new: {canon(nv)}")
    w(f"\nveiled-related among the {len(modtext_examples)} collected examples: {veiled_involved}")
w(f"\n## icon / properties\n")
w(f"- `icon` differs on **{icon_diff}** items.")
w(f"- `properties` differs on **{props_diff}** items.\n")
txt = "\n".join(L)
print(txt)
print(txt)
