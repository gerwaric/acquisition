#!/usr/bin/env python3
"""Extract the trade site's query grammar into data/grammar.json.

Inputs (MANIFEST.md): data/filters-<date>.json, data/stats-<date>.json,
data/items-<date>.json (committed) and the page bundle under raw/
(local only: trade.*.js for the stat group types and the request shape,
main.*.js for the property-type table). Fails loudly without raw/ —
the output is regenerable only with the capture that produced it.

Nothing here is inferred: every value is read out of a file, and the
`source` field on each section names which one. The gap assessment
(local / none / added) is a finding and lives in README.md, not here.
"""
import glob, json, re, sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
date = sys.argv[1] if len(sys.argv) > 1 else "2026-09-12"

def load(name):
    return json.load(open(root / "data" / f"{name}-{date}.json"))

def bundle(prefix):
    hits = glob.glob(str(root / "raw" / "Trade - Path of Exile_files" / f"{prefix}.*.js"))
    if len(hits) != 1:
        sys.exit(f"need exactly one raw/…/{prefix}.*.js (found {len(hits)}); the page save is local only — see MANIFEST.md")
    return Path(hits[0]).name, open(hits[0], encoding="utf-8", errors="replace").read()

def module(src, name):
    i = src.find(f'define("{name}"')
    if i < 0:
        sys.exit(f"module {name} not found")
    j = src.find('define("PoE/', i + 10)
    return src[i:j if j > 0 else len(src)]

trade_name, trade = bundle("trade")
main_name, main = bundle("main")

# --- filter groups: data/filters.json verbatim, value shape from the flags
filters = load("filters")["result"]
SHAPE_FLAGS = ("minMax", "option", "sockets", "input")
groups = []
for g in filters:
    fl = []
    for f in g["filters"]:
        shape = [k for k in SHAPE_FLAGS if k in f]
        entry = {"id": f["id"], "text": f.get("text"), "shape": shape}
        if "tip" in f:
            entry["tip"] = f["tip"]
        if "option" in f:
            opts = f["option"].get("options")
            if opts is None:  # an option object without a list: recorded verbatim
                entry["option_raw"] = f["option"]
            else:
                entry["options"] = [o.get("id") for o in opts]
                entry["option_texts"] = {str(o.get("id")): o.get("text") for o in opts}
        if "input" in f:
            entry["input"] = f["input"]
        fl.append(entry)
    groups.append({"id": g["id"], "title": g.get("title"), "filters": fl})

# --- stat group types: PoE/Trade/Data/Static in the trade bundle, verbatim tips
static_mod = module(trade, "PoE/Trade/Data/Static")
sg_src = static_mod[static_mod.find("statGroups:[") + len("statGroups:") :]
sg_src = sg_src[: sg_src.find("]}") + 1]
stat_groups = []
for m in re.finditer(r'\{type:"([a-z0-9]+)",title:e\.translate\("([^"]+)"\)(.*?)\}(?=,\{type:|\]$)', sg_src, re.S):
    typ, title, rest = m.groups()
    tip = re.search(r'tip:e\.translate\("((?:[^"\\]|\\.)*)"\)', rest)
    stat_groups.append({
        "type": typ,
        "title": title,
        "tip": tip.group(1).replace("\\n", "\n") if tip else None,
        "group_value": "min_max" if "minMax:!0" in rest else ("min_only" if "minOnly:!0" in rest else None),
        "weight": "weight:!0" in rest,
        "mutable": "mutable:!1" not in rest,
    })
limits = {
    "resultLimit": int(re.search(r"resultLimit:(\d+)", static_mod).group(1)),
    "liveResultTotalLimit": int(re.search(r"liveResultTotalLimit:(\d+)", static_mod).group(1)),
}

# --- request shape: PoE/Trade/Service and the App's query() computed, read not paraphrased
service = module(trade, "PoE/Trade/Service")
search_url = re.search(r'performSearch:function\(t,e,n,i\)\{var r=\{url:this\.apiUrl\(([^)]*)\)', service).group(1)
qfun_i = trade.find("query:function(){var t={};")
qfun = trade[qfun_i : trade.find("return t},", qfun_i) + 9]
default_sort = re.search(r'sort:"search"===this\.state\.tab\?(\{[^}]*\}):(\{[^}]*\})', trade)
state_i = trade.find("E={state:{id:null,tab:")
persisted_state = trade[state_i + 2 : trade.find("},mutations:", state_i) + 1]

# --- the property-type → field table: the popup's typeToField helper in main
ttf = re.search(r'registerHelper\("typeToField",\(function\(e\)\{var n=(\[.*?\])\[e\.type\]', module(main, "PoE/Item/Popup"), re.S).group(1)
ttf = json.loads(ttf.replace("null", "null"))
type_to_field = {str(i): v for i, v in enumerate(ttf) if v is not None}

# --- the renderer's mod arrays, in display order, and the hash-category → css mapping
popup = module(main, "PoE/Item/Popup")
mod_arrays = re.findall(r'o\("([a-zA-Z]+Mods)"\)', popup)
item_component = module(trade, "PoE/Trade/Component/Item")
hash_map = re.search(r"_\.each\((\{implicit:.*?\}),\(function", item_component).group(1)
hash_to_css = dict(re.findall(r'([a-z]+):"([^"]+)"', hash_map))
search_by_me_cats = re.search(r'searchByMe:function\(\)\{.*?(\[(?:"[a-z]+",?)+\])\.forEach', trade, re.S).group(1)

stats = load("stats")["result"]
items = load("items")["result"]

out = {
    "_generated": "by scripts/extract-grammar.py from the files in MANIFEST.md — do not edit",
    "captured": date,
    "sources": {
        "filters": f"data/filters-{date}.json",
        "stats": f"data/stats-{date}.json",
        "items": f"data/items-{date}.json",
        "trade_bundle": f"raw/Trade - Path of Exile_files/{trade_name}",
        "main_bundle": f"raw/Trade - Path of Exile_files/{main_name}",
    },
    "request": {
        "source": "PoE/Trade/Service.performSearch; PoE/Trade/App.query(); addSearch",
        "search_url_expression": search_url,
        "search_url": "POST /api/trade/search[/<realm>]/<league>  (pc by omission)",
        "body": "{query, sort}",
        "default_sort": json.loads(re.sub(r"(\w+):", r'"\1":', default_sort.group(1))) if default_sort else None,
        "persisted_state_source": persisted_state,
        "query_assembly_source": qfun,
        "limits": limits,
    },
    "query_shape": {
        "source": "read from query_assembly_source above",
        "status": "{option: <status id>}",
        "term": "free text; when set, name and type are not sent",
        "name": "string, or {option: <name>, discriminator: <disc>}",
        "type": "string, or {option: <type>, discriminator: <disc>}",
        "stats": "[{type: <stat group type>, filters: [{id, value: {min?, max?, weight?, option?}, disabled}], value: {min?, max?}?, disabled?}]",
        "filters": "{<group id>: {filters: {<filter id>: {min?, max?} | {option} | {r?, g?, b?, w?, min?, max?} | <input>}, disabled?}} — a group is sent only when it has filters",
    },
    "filter_groups": groups,
    "stat_groups": stat_groups,
    "stat_categories": [{"id": c["id"], "label": c["label"], "entries": len(c["entries"])} for c in stats],
    "item_categories": [{"id": c["id"], "label": c["label"], "entries": len(c["entries"])} for c in items],
    "property_type_to_field": {"source": "PoE/Item/Popup typeToField (main bundle)", "table": type_to_field},
    "renderer_mod_arrays_in_order": {"source": "PoE/Item/Popup processContext (main bundle)", "arrays": mod_arrays},
    "hash_category_to_css_class": {"source": "PoE/Trade/Component/Item render (trade bundle)", "map": hash_to_css},
    "search_by_me_categories": json.loads(search_by_me_cats),
}
dest = root / "data" / "grammar.json"
dest.write_text(json.dumps(out, indent=1, ensure_ascii=False) + "\n")
print("wrote", dest, dest.stat().st_size, "bytes;", len(groups), "groups,", sum(len(g["filters"]) for g in groups), "filters,", len(stat_groups), "stat group types,", len(type_to_field), "property types")
