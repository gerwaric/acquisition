#!/usr/bin/env python3
"""The owner's captures as a scrubbed extract: ../data/captures.json.

Reads ../raw/searches/<search>-search.json and <search>-fetch.json, with any
further page of the same search as <search>-fetch-2.json, -3 and so on (local
only: a fetch carries seller accounts and tokens; MANIFEST.md) and writes,
per search: the query as the site returned it (the id, decoded), whether it
is the sheet's query, the response's counts, and for each fetched item only
what the evidence reads — its names, rarity, properties, the site's computed
`extended` figures, and every line with its domain, hash, flags and mods.
No item id, icon, seller, price, stash position or token leaves raw/.

    captures.py             # write ../data/captures.json
    captures.py --manifest  # print the manifest's rows for what is in raw/

The site rewrites an id on the way back: `"disabled":false` is dropped and
keys are reordered. A returned query is the sheet's when the two are equal
once every `"disabled": false` is removed from both.
"""

import base64
import csv
import gzip
import hashlib
import json
import sys
import time
from pathlib import Path

TRACK = Path(__file__).resolve().parents[1]
RAW = TRACK / "raw" / "searches"
SHEET = TRACK / "data" / "search-sheet.csv"
OUT = TRACK / "data" / "captures.json"

ARRAYS = [
    "enchantMods", "implicitMods", "fracturedMods", "explicitMods", "craftedMods",
    "scourgeMods", "crucibleMods", "utilityMods", "veiledMods", "mutatedMods",
    "pseudoMods",
]
SCRUBBED = ("whisper", "hideout_token", '"account": {', '"icon": "', '"stash": {', '"price": {')


def decode(search_id):
    padded = search_id + "=" * (-len(search_id) % 4)
    return json.loads(gzip.decompress(base64.urlsafe_b64decode(padded)))


def bare(node):
    """A query with every `"disabled": false` removed."""
    if isinstance(node, dict):
        return {k: bare(v) for k, v in node.items() if not (k == "disabled" and v is False)}
    if isinstance(node, list):
        return [bare(v) for v in node]
    return node


def line(entry):
    if not isinstance(entry, dict):
        return {"description": entry}
    return {
        "description": entry["description"],
        "domain": entry.get("domain"),
        "hash": entry.get("hash"),
        "flags": entry.get("flags"),
        "mods": [
            {k: m.get(k) for k in ("name", "tier", "level", "magnitudes")}
            for m in entry.get("mods") or []
        ],
    }


def item(raw):
    extended = raw.get("extended", {})
    return {
        "name": raw.get("name"),
        "typeLine": raw.get("typeLine"),
        "baseType": raw.get("baseType"),
        "rarity": raw.get("rarity"),
        "frameType": raw.get("frameType"),
        "properties": [
            [p.get("type"), p.get("name"), [v[0] for v in p.get("values", [])]]
            for p in raw.get("properties", [])
        ],
        "extended": {k: v for k, v in extended.items() if k not in ("hashes", "text")},
        "lines": {a: [line(e) for e in raw[a]] for a in ARRAYS if a in raw},
    }


def searches():
    return sorted(p.name[: -len("-search.json")] for p in RAW.glob("*-search.json"))


def manifest():
    for path in sorted(RAW.glob("*.json")):
        body = path.read_bytes()
        stamp = time.strftime("%Y-%m-%d %H:%M", time.localtime(path.stat().st_mtime))
        print(f"| `raw/searches/{path.name}` | {len(body)} | {stamp} | `{hashlib.sha256(body).hexdigest()}` |")
    return 0


def main():
    if sys.argv[1:] == ["--manifest"]:
        return manifest()
    if sys.argv[1:]:
        print(__doc__)
        return 2
    with SHEET.open(newline="\n") as f:
        sheet = {r["search"]: json.loads(r["query"]) for r in csv.DictReader(f)}
    out = {
        "_generated": "by scripts/captures.py from raw/searches (MANIFEST.md) — do not edit",
        "searches": {},
    }
    for name in searches():
        answer = json.loads((RAW / f"{name}-search.json").read_text())
        returned = decode(answer["id"])
        entry = {
            "returned_query": returned,
            "sheet_query": (
                "no such row in the sheet" if name not in sheet
                else "the same" if bare(returned) == bare(sheet[name])
                else "differs"
            ),
            "search": {k: v for k, v in answer.items() if k not in ("id", "result")},
            "ids_returned": len(answer.get("result", [])),
            "fetched": [],
        }
        pages = [RAW / f"{name}-fetch.json"] + sorted(
            RAW.glob(f"{name}-fetch-*.json"), key=lambda p: int(p.stem.rsplit("-", 1)[1])
        )
        for fetch in pages:
            if fetch.exists() and fetch.stat().st_size:
                entry["fetched"] += [item(r["item"]) for r in json.loads(fetch.read_text())["result"]]
        out["searches"][name] = entry
        print(f"{name}: sheet query {entry['sheet_query']}; total {answer.get('total')}; "
              f"{entry['ids_returned']} ids; {len(entry['fetched'])} items fetched")
    text = json.dumps(out, indent=1, ensure_ascii=False) + "\n"
    for bad in SCRUBBED:
        if bad in text:
            raise SystemExit(f"scrub guard: {bad!r} would leave raw/")
    OUT.write_text(text)
    print(f"wrote {OUT.relative_to(TRACK.parent.parent)}, {len(text.encode())} bytes; scrub guard passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
