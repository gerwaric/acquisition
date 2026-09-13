#!/usr/bin/env python3
"""What the trade site's `fetch` responses carry, as a scrubbed census.

Reads raw/searches/q*-fetch.json (local only: they hold seller accounts and
tokens) and writes data/fetch-census.json: the request bodies verbatim, the
search responses minus their result ids, and for the fetched items only
shape and evidence — key sets, domains, flags, `extended` fields, and the
mod lines with their contributing mod names, tiers and magnitudes. No item
id, icon, seller, price, stash position or token leaves raw/. Item names
are the game's random rare names; base types are public data.
"""
import collections, glob, json, re, sys
from pathlib import Path
root = Path(__file__).resolve().parent.parent
ARR = ["implicitMods", "explicitMods", "craftedMods", "enchantMods", "fracturedMods", "scourgeMods",
       "utilityMods", "pseudoMods", "veiledMods", "crucibleMods", "ultimatumMods", "logbookMods", "mutatedMods"]
out = {"_generated": "by scripts/fetch-census.py from raw/searches (MANIFEST.md) — do not edit", "queries": {}}
tot = {"items": 0, "lines": 0, "arrays": collections.Counter(), "line_keys": collections.Counter(), "mod_keys": collections.Counter(),
       "domains": collections.Counter(), "flags": collections.Counter(), "extended_keys": collections.Counter(),
       "lines_fed_by_several_mods": [], "mods_feeding_several_lines": 0, "item_keys": collections.Counter()}
for req in sorted(glob.glob(str(root / "raw/searches/q*-request.json"))):
    q = Path(req).name.split("-")[0]
    entry = {}
    body = open(req).read()
    entry["request"] = json.loads(body) if body.strip() else "saved empty"
    s = open(root / f"raw/searches/{q}-search.json").read()
    if s.strip():
        sd = json.loads(s); entry["search"] = {k: (len(v) if k == "result" else v) for k, v in sd.items() if k != "id"}
        entry["search"]["result"] = f"{len(sd['result'])} ids"
    else:
        entry["search"] = "saved empty"
    f = open(root / f"raw/searches/{q}-fetch.json").read()
    items = []
    if f.strip():
        for r in json.loads(f)["result"]:
            it = r["item"]; tot["items"] += 1
            tot["item_keys"][tuple(sorted(it.keys()))] += 1
            ex = it.get("extended", {})
            tot["extended_keys"][tuple(sorted(k for k in ex if k not in ("hashes", "text")))] += 1
            rec = {"typeLine": it.get("typeLine"), "name": it.get("name"), "rarity": it.get("rarity"), "frameType": it.get("frameType"),
                   "properties": [(p.get("type"), p.get("name"), [v[0] for v in p.get("values", [])]) for p in it.get("properties", [])],
                   "extended": {k: v for k, v in ex.items() if k not in ("hashes", "text")},
                   "hashes": ex.get("hashes"), "lines": {}}
            seen = collections.defaultdict(list)
            for a in ARR:
                if a not in it: continue
                tot["arrays"][(a, type(it[a][0]).__name__ if it[a] else "empty")] += 1
                ls = []
                for l in it[a]:
                    if not isinstance(l, dict):
                        ls.append(l); continue
                    tot["lines"] += 1; tot["line_keys"][tuple(sorted(l))] += 1
                    tot["domains"][(a, l.get("domain"))] += 1
                    for k, v in (l.get("flags") or {}).items(): tot["flags"][(a, k, str(v))] += 1
                    mods = l.get("mods") or []
                    for m in mods:
                        tot["mod_keys"][tuple(sorted(m))] += 1
                        if m.get("name"): seen[(m["name"], m.get("tier"))].append(l["description"])
                    if len(mods) > 1:
                        tot["lines_fed_by_several_mods"].append([q, it.get("typeLine"), l["description"], [[m.get("name"), m.get("tier")] for m in mods]])
                    ls.append({"description": l["description"], "domain": l.get("domain"), "hash": l.get("hash"), "flags": l.get("flags"),
                               "mods": [{k: m.get(k) for k in ("name", "tier", "level", "magnitudes")} for m in mods]})
                rec["lines"][a] = ls
            tot["mods_feeding_several_lines"] += sum(1 for v in seen.values() if len(v) > 1)
            items.append(rec)
    entry["fetched"] = items
    out["queries"][q] = entry
def ctr(c): return [[list(k) if isinstance(k, tuple) else k, v] for k, v in sorted(c.items(), key=lambda kv: -kv[1])]
out["census"] = {k: (ctr(v) if isinstance(v, collections.Counter) else v) for k, v in tot.items()}
dest = root / "data" / "fetch-census.json"
dest.write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n")
c = out["census"]
print("wrote", dest, dest.stat().st_size, "bytes; items", c["items"], "lines", c["lines"], "multi-mod lines", len(c["lines_fed_by_several_mods"]), "hybrid mods", c["mods_feeding_several_lines"])
# the guard: nothing personal in the output
s = dest.read_text()
for bad in ("whisper", "hideout_token", '"account": {', '"icon": "', '"stash": {', '"price": {'):  # listing objects; the request's sort key and stat ids are fine
    assert bad not in s, bad
print("scrub guard passed")
